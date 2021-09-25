use anyhow::Result;
use std::env;
use strum_macros::EnumString;

#[derive(Debug, PartialEq, Clone, Copy, EnumString)]
pub enum CIKind {
    #[strum(serialize = "gitlabci")]
    GitLab,
}

#[derive(Debug, Clone)]
pub struct CI {
    url: String,
    merge_request: MergeRequest,
}

impl CI {
    pub fn new(ci: CIKind) -> Result<Self> {
        match ci {
            CIKind::GitLab => {
                // todo: make this as function
                let url = env::var("CI_JOB_URL")?;
                let number = env::var("CI_MERGE_REQUEST_IID")?.parse()?;
                let revision = env::var("CI_COMMIT_SHA")?;
                let merge_request = MergeRequest { number, revision };
                Ok(Self { url, merge_request })
            }
        }
    }

    pub fn url(&self) -> &String {
        &self.url
    }

    pub fn merge_request(&self) -> &MergeRequest {
        &self.merge_request
    }
}

#[derive(Debug, Clone)]
pub struct MergeRequest {
    number: u64,
    revision: String,
}

impl MergeRequest {
    pub fn number(&self) -> &u64 {
        &self.number
    }
}
