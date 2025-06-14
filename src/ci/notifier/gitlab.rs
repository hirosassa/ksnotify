use crate::ci::MergeRequest;
use crate::template::Template;

use anyhow::Result;
use gitlab::Gitlab;
use gitlab::api::projects::merge_requests::notes::{
    CreateMergeRequestNote, EditMergeRequestNote, MergeRequestNotes,
};
use gitlab::api::projects::repository::commits::MergeRequests;
use gitlab::api::{self, Query};
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
    merge_request: MergeRequest,
    job_url: String,
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
    pub fn new() -> Result<Self> {
        info!("create GitLab client");

        let base_url = Self::get_base_url()?;
        let token = Self::get_token()?;

        let client = Gitlab::new(base_url, token)?;
        let project = Self::get_project()?;
        let merge_request = Self::get_merge_request()?;
        let job_url = Self::get_job_url()?;
        Ok(Self {
            client,
            project,
            merge_request,
            job_url,
        })
    }

    fn get_merge_request() -> Result<MergeRequest> {
        let number = env::var("CI_MERGE_REQUEST_IID").ok();
        let number = if number.is_some() {
            Some(number.unwrap().parse::<u64>()?)
        } else {
            None
        };
        let commit_sha = env::var("CI_COMMIT_SHA")?;
        Ok(MergeRequest { number, commit_sha })
    }

    fn get_job_url() -> Result<String> {
        Ok(env::var("CI_JOB_URL")?)
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

    fn retrieve_same_build_comment(&self, template: &Template) -> Result<Option<Note>> {
        info!("retrieve same build comment");
        let endpoint = MergeRequestNotes::builder()
            .project(self.project)
            .merge_request(self.retrieve_merge_request_iid_with_fallback(self.merge_request())?)
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

    const fn merge_request(&self) -> &MergeRequest {
        &self.merge_request
    }
}

impl Notifiable for GitlabNotifier {
    fn notify(&self, template: &Template, patch: bool) -> Result<()> {
        info!("notify to GitLab");

        let same_build_comment = self.retrieve_same_build_comment(template)?;

        // update comment if existed
        if patch {
            if let Some(same_build_comment) = same_build_comment {
                let note = EditMergeRequestNote::builder()
                    .project(self.project)
                    .merge_request(
                        self.retrieve_merge_request_iid_with_fallback(self.merge_request())?,
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
            .merge_request(self.retrieve_merge_request_iid_with_fallback(self.merge_request())?)
            .body(template.render()?)
            .build()
            .map_err(anyhow::Error::msg)?;
        api::ignore(note).query(&self.client)?;
        Ok(())
    }

    fn job_url(&self) -> String {
        self.job_url.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_project() {
        temp_env::with_vars([("CI_PROJECT_ID", Some("123"))], || {
            let project = GitlabNotifier::get_project().unwrap();
            assert_eq!(project, 123);
        });
    }

    #[test]
    fn test_get_job_url() {
        temp_env::with_var("CI_JOB_URL", Some("https://example.com/ksnotify"), || {
            let job_url = GitlabNotifier::get_job_url().unwrap();
            assert_eq!(job_url, "https://example.com/ksnotify");
        });
    }

    #[test]
    fn test_get_merge_request() {
        temp_env::with_vars(
            [
                ("CI_MERGE_REQUEST_IID", Some("123")),
                ("CI_COMMIT_SHA", Some("abcdefg")),
            ],
            || {
                let merge_request = GitlabNotifier::get_merge_request().unwrap();
                assert_eq!(merge_request.number, Some(123));
                assert_eq!(merge_request.commit_sha, "abcdefg");
            },
        );
    }

    #[test]
    fn test_get_merge_request_without_number() {
        temp_env::with_vars(
            [
                ("CI_MERGE_REQUEST_IID", None),
                ("CI_COMMIT_SHA", Some("abcdefg")),
            ],
            || {
                let merge_request = GitlabNotifier::get_merge_request().unwrap();
                assert_eq!(merge_request.number, None);
                assert_eq!(merge_request.commit_sha, "abcdefg");
            },
        );
    }
}
