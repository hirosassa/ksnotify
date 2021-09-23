use anyhow::Result;
use gitlab::api::{self, projects::merge_requests::notes::CreateMergeRequestNote, Query};
use gitlab::Gitlab;
use yaml_rust::Yaml;

use super::Notifiable;
use super::super::CI;

#[derive(Debug)]
struct Repository {
    owner: String,
    project: String,
}

#[derive(Debug)]
pub struct GitlabNotifier {
    client: Gitlab,
    repository: Repository,
    ci: CI,
}

impl GitlabNotifier {
    pub fn new(ci: CI, config: Yaml) -> Result<Self> {
        let client = Gitlab::new(
            config["base_url"]
                .as_str()
                .expect("failed to load base_url of GitLab config")
                .to_string(),
            config["token"]
                .as_str()
                .expect("failed to load token of GitLab config")
                .to_string(),
        )?;
        let repository = Repository {
            owner: config["repository"]["owner"]
                .as_str()
                .expect("failed to load the owner of the repository")
                .to_string(),
            project: config["repository"]["project"]
                .as_str()
                .expect("failed to load the project name of the repository")
                .to_string(),
        };
        Ok(Self {
            client,
            repository,
            ci,
        })
    }
}

impl Notifiable for GitlabNotifier {
    fn notify(&self, body: String) -> Result<()> {
        let project = format!("{}/{}", self.repository.owner, self.repository.project);
        let note = CreateMergeRequestNote::builder()
            .project(project)
            .merge_request(self.ci.merge_request.number)
            .body(body)
            .build()
            .map_err(anyhow::Error::msg)?;
        api::ignore(note).query(&self.client)?;
        Ok(())
    }
}
