use crate::ci::CI;
use crate::template::Template;

use anyhow::{Context, Result};
use gitlab::api::projects::merge_requests::notes::{EditMergeRequestNote, MergeRequestNotes};
use gitlab::api::{self, projects::merge_requests::notes::CreateMergeRequestNote, Query};
use gitlab::types::Note;
use gitlab::Gitlab;
use log::info;
use std::env;

use super::Notifiable;

const ENV_GITLAB_TOKEN: &str = "KSNOTIFY_GITLAB_TOKEN";

#[derive(Debug)]
pub struct GitlabNotifier {
    client: Gitlab,
    project: u64,
    ci: CI,
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
            .merge_request(*self.ci.merge_request().number())
            .build()
            .map_err(anyhow::Error::msg)?;
        let comments: Vec<Note> = endpoint.query(&self.client)?;

        for comment in comments {
            if template.is_same_build(&comment.body)? {
                return Ok(Some(comment));
            }
        }
        Ok(None)
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
                    .merge_request(*self.ci.merge_request().number())
                    .note(same_build_comment.id.value())
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
            .merge_request(*self.ci.merge_request().number())
            .body(template.render()?)
            .build()
            .map_err(anyhow::Error::msg)?;
        api::ignore(note).query(&self.client)?;
        Ok(())
    }
}
