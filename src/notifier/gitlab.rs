use crate::ci::{MergeRequest, CI};
use crate::template::Template;

use anyhow::{Context, Result};
use gitlab::api::projects::merge_requests::notes::{
    CreateMergeRequestNote, EditMergeRequestNote, MergeRequestNotes,
};
use gitlab::api::projects::repository::commits::MergeRequests;
use gitlab::api::{self, Query};
use gitlab::Gitlab;
use log::info;
use serde::Deserialize;
use std::env;

use super::Notifiable;

const ENV_GITLAB_TOKEN: &str = "KSNOTIFY_GITLAB_TOKEN";
const LIST_NOTES_LIMIT: usize = 300;
const LIST_MERGE_REQUESTS_LIMIT: usize = 100;

#[derive(Debug)]
pub struct GitlabNotifier {
    client: Gitlab,
    project: u64,
    ci: CI,
}

#[derive(Debug, Deserialize)]
struct Note {
    id: u64,
    body: String,
}

#[derive(Debug, Deserialize)]
struct GitLabMergeRequest {
    iid: u64,
}

impl GitlabNotifier {
    pub fn new(ci: &CI) -> Result<Self> {
        info!("create GitLab client");

        let base_url = Self::get_base_url()?;
        let token = Self::get_token()?;
        let client =
            Gitlab::new(base_url, token).with_context(|| "failed to create client".to_string())?;
        let project = Self::get_project()?;
        Ok(Self {
            client,
            project,
            ci: ci.clone(),
        })
    }

    fn get_token() -> Result<String> {
        Ok(env::var(ENV_GITLAB_TOKEN)?)
    }

    fn get_base_url() -> Result<String> {
        Ok(env::var("CI_SERVER_HOST")?)
    }

    fn get_project() -> Result<u64> {
        Ok(env::var("CI_PROJECT_ID")?.parse::<u64>()?)
    }

    fn retrive_same_build_comment(&self, template: &Template) -> Result<Option<Note>> {
        info!("retrieve same build comment");
        let endpoint = MergeRequestNotes::builder()
            .project(self.project)
            .merge_request(self.retrieve_merge_request_iid_with_fallback(self.ci.merge_request())?)
            .build()
            .map_err(anyhow::Error::msg)?;
        let comments: Vec<Note> = api::paged(endpoint, api::Pagination::Limit(LIST_NOTES_LIMIT))
            .query(&self.client)
            .map_err(anyhow::Error::msg)?;

        for comment in comments {
            if template.is_same_build(&comment.body)? {
                return Ok(Some(comment));
            }
        }
        Ok(None)
    }

    /// Retrieve merge request IID with fallback.
    /// If merge request number is not provided, it will retrieve the merge request IID by commit SHA.
    fn retrieve_merge_request_iid_with_fallback(&self, mr: &MergeRequest) -> Result<u64> {
        if let Some(number) = mr.number {
            return Ok(number);
        }

        let endpoint = MergeRequests::builder()
            .project(self.project)
            .sha(mr.commit_sha.clone())
            .build()
            .map_err(anyhow::Error::msg)?;
        let mrs: Vec<GitLabMergeRequest> =
            api::paged(endpoint, api::Pagination::Limit(LIST_MERGE_REQUESTS_LIMIT))
                .query(&self.client)
                .map_err(anyhow::Error::msg)?;
        if mrs.is_empty() {
            return Err(anyhow::anyhow!("no merge request found"));
        }
        Ok(mrs[0].iid)
    }
}

impl Notifiable for GitlabNotifier {
    fn notify(&self, template: Template, patch: bool) -> Result<()> {
        info!("notify to GitLab");

        let same_build_comment = self.retrive_same_build_comment(&template)?;

        // update comment if existed
        if patch {
            if let Some(same_build_comment) = same_build_comment {
                let note = EditMergeRequestNote::builder()
                    .project(self.project)
                    .merge_request(
                        self.retrieve_merge_request_iid_with_fallback(self.ci.merge_request())?,
                    )
                    .note(same_build_comment.id)
                    .body(template.render()?)
                    .build()
                    .map_err(anyhow::Error::msg)?;
                api::ignore(note).query(&self.client)?;
                return Ok(());
            }
        }

        // create new comment
        let note = CreateMergeRequestNote::builder()
            .project(self.project)
            .merge_request(self.retrieve_merge_request_iid_with_fallback(self.ci.merge_request())?)
            .body(template.render()?)
            .build()
            .map_err(anyhow::Error::msg)?;
        api::ignore(note).query(&self.client)?;
        Ok(())
    }
}
