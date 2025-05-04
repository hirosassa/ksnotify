use crate::template::Template;

use log::{debug, info};
use octocrab::{models::issues::Comment, Octocrab};
use std::env;

use super::Notifiable;
use anyhow::Result;

#[derive(Debug)]
pub struct GithubNotifier {
    owner: String,
    repo: String,
    pr_number: u64,
    client: Octocrab,
}

impl GithubNotifier {
    pub fn new() -> Result<Self> {
        info!("create GitHub client");

        let (owner, repo) = Self::get_repository()?;
        let pr_number = Self::get_pr_number()?;
        let client = Octocrab::builder()
            .personal_token(env::var("GITHUB_TOKEN")?)
            .build()?;
        debug!(
            "GitHub client created: {}, {}, {}, {:?}",
            owner, repo, pr_number, client
        );

        Ok(Self {
            owner,
            repo,
            pr_number,
            client,
        })
    }

    // Default environment variables in GitHub Actions
    // see: https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/store-information-in-variables#default-environment-variables
    fn get_repository() -> Result<(String, String)> {
        // GITHUB_REPOSITORY is like <owner>/<repo>
        let env = env::var("GITHUB_REPOSITORY")?;
        let parts = env.split('/').collect::<Vec<&str>>();
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    fn get_pr_number() -> Result<u64> {
        // GITHUB_REF_NAME is like <pr_number>/merge
        let env = env::var("GITHUB_REF_NAME")?;
        let parts = env.split('/').next().unwrap();
        Ok(parts.parse::<u64>()?)
    }

    async fn post_comment(&self, template: &Template, patch: bool) -> Result<()> {
        if patch {
            if let Some(same_build_comment) = self.retrive_same_build_comment(template).await? {
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
        let _ = self
            .client
            .issues(&self.owner, &self.repo)
            .create_comment(self.pr_number, template.render()?)
            .await?;
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

    async fn retrive_same_build_comment(&self, template: &Template) -> Result<Option<Comment>> {
        info!("retrieve same build comment");

        // get recent 300 comments from the PR
        let comments = self
            .client
            .issues(&self.owner, &self.repo)
            .list_comments(self.pr_number)
            .per_page(100) // 100 comments per page (GitHub API limitation)
            .page(3u32) // 3 pages * 100 comments = 300 comments
            .send()
            .await?;

        for comment in comments {
            if let Some(body) = comment.body.clone() {
                if template.is_same_build(&body)? {
                    return Ok(Some(comment));
                }
            }
        }
        Ok(None)
    }
}

impl Notifiable for GithubNotifier {
    fn notify(&self, template: Template, patch: bool) -> Result<()> {
        info!("notify to GitHub");
        smol::block_on(self.post_comment(&template, patch))?;
        Ok(())
    }
}
