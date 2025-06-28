use crate::ci::MergeRequest;
use crate::template::Template;

use log::{debug, info};
use octocrab::{Octocrab, models::issues::Comment};
use std::env;

use super::Notifiable;
use anyhow::Result;

#[derive(Debug)]
pub struct GithubNotifier {
    runtime: tokio::runtime::Runtime,
    client: Octocrab,
    owner: String,
    repo: String,
    pull_request: MergeRequest,
    job_url: String,
}

impl GithubNotifier {
    pub fn new() -> Result<Self> {
        info!("create GitHub client");

        let token = Self::get_token()?;
        let (owner, repo) = Self::get_repository()?;
        let pull_request = Self::get_pull_request()?;
        let job_url = Self::get_job_url()?;
        debug!("owner: {owner}, repo: {repo}, pull_request: {pull_request:?}");

        // octocrab needs tokio runtime
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let client =
            runtime.block_on(async { Octocrab::builder().personal_token(token).build() })?;
        Ok(Self {
            runtime,
            client,
            owner,
            repo,
            pull_request,
            job_url,
        })
    }

    fn get_pull_request() -> Result<MergeRequest> {
        // GITHUB_REF_NAME is like <number>/merge
        let ref_name = env::var("GITHUB_REF_NAME").expect("GITHUB_REF_NAME must be set.");
        let number = if ref_name.ends_with("/merge") {
            Some(ref_name.split("/").next().unwrap().parse::<u64>()?)
        } else {
            None
        };
        let commit_sha = env::var("GITHUB_SHA").expect("GITHUB_SHA must be set.");

        Ok(MergeRequest { number, commit_sha })
    }

    // Default environment variables in GitHub Actions
    // see: https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/store-information-in-variables#default-environment-variables
    fn get_job_url() -> Result<String> {
        let repository = env::var("GITHUB_REPOSITORY").expect("GITHUB_REPOSITORY must be set.");
        let run_id = env::var("GITHUB_RUN_ID").expect("GITHUB_RUN_ID must be set.");
        Ok(format!(
            "https://github.com/{repository}/actions/runs/{run_id}"
        ))
    }

    fn get_token() -> Result<String> {
        Ok(env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN must be set."))
    }

    fn get_repository() -> Result<(String, String)> {
        // GITHUB_REPOSITORY is like <owner>/<repo>
        let env = env::var("GITHUB_REPOSITORY").expect("GITHUB_REPOSITORY must be set.");
        let parts = env.split('/').collect::<Vec<&str>>();
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    async fn post_comment(&self, template: &Template, patch: bool) -> Result<()> {
        if patch {
            if let Some(same_build_comment) = self.retrieve_same_build_comment(template).await? {
                let _ = self
                    .update_existing_comment(template, same_build_comment)
                    .await;
                return Ok(());
            }
        }

        let _ = self.create_new_comment(template).await;
        Ok(())
    }

    async fn create_new_comment(&self, template: &Template) -> Result<()> {
        let pr_number = if self.pull_request.number.is_some() {
            self.pull_request.number.unwrap()
        } else {
            debug!("pull request number is None");
            return Ok(());
        };

        debug!(
            "create new comment for PR #{}, owner:{}, repo: {}",
            pr_number, self.owner, self.repo
        );
        let res = self
            .client
            .issues(&self.owner, &self.repo)
            .create_comment(pr_number, template.render()?)
            .await;
        debug!("create comment response: {res:?}");
        res?;
        Ok(())
    }

    async fn update_existing_comment(&self, template: &Template, comment: Comment) -> Result<()> {
        let _ = self
            .client
            .issues(&self.owner, &self.repo)
            .update_comment(comment.id, template.render()?)
            .await?;
        Ok(())
    }

    async fn retrieve_same_build_comment(&self, template: &Template) -> Result<Option<Comment>> {
        info!("retrieve same build comment");

        let pr_number = if self.pull_request.number.is_some() {
            self.pull_request.number.unwrap()
        } else {
            debug!("pull request number is None");
            return Ok(None);
        };

        // get recent 300 comments from the PR
        let comments = self
            .client
            .issues(&self.owner, &self.repo)
            .list_comments(pr_number)
            .send()
            .await?;
        if comments.items.is_empty() {
            debug!("no comments found in the PR");
            return Ok(None);
        }

        for comment in comments.items {
            if let Some(body) = comment.body.clone() {
                debug!("checking comment: {body}");
                if template.is_same_build(&body)? {
                    return Ok(Some(comment));
                }
            }
        }
        Ok(None)
    }
}

impl Notifiable for GithubNotifier {
    fn notify(&self, template: &Template, patch: bool) -> Result<()> {
        info!("notify to GitHub");
        self.runtime.block_on(self.post_comment(template, patch))?;
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
    fn test_get_repository() {
        temp_env::with_var("GITHUB_REPOSITORY", Some("owner/repo"), || {
            let (owner, repo) = GithubNotifier::get_repository().unwrap();
            assert_eq!(owner, "owner");
            assert_eq!(repo, "repo");
        });
    }

    #[test]
    fn test_get_job_url() {
        temp_env::with_vars(
            [
                ("GITHUB_REPOSITORY", Some("owner/repo")),
                ("GITHUB_RUN_ID", Some("12345")),
            ],
            || {
                let job_url = GithubNotifier::get_job_url().unwrap();
                assert_eq!(job_url, "https://github.com/owner/repo/actions/runs/12345");
            },
        );
    }

    #[test]
    fn test_get_pull_request() {
        temp_env::with_vars(
            [
                ("GITHUB_REF_NAME", Some("123/merge")),
                ("GITHUB_SHA", Some("abc123")),
            ],
            || {
                let pull_request = GithubNotifier::get_pull_request().unwrap();
                assert_eq!(pull_request.number, Some(123));
                assert_eq!(pull_request.commit_sha, "abc123");
            },
        );
    }

    #[test]
    fn test_get_pull_request_without_number() {
        temp_env::with_vars(
            [
                ("GITHUB_REF_NAME", Some("feature-branch")),
                ("GITHUB_SHA", Some("abc123")),
            ],
            || {
                let pull_request = GithubNotifier::get_pull_request().unwrap();
                assert_eq!(pull_request.number, None);
                assert_eq!(pull_request.commit_sha, "abc123");
            },
        );
    }
}
