use anyhow::Result;
use std::env;
use strum_macros::EnumString;

#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumString)]
pub enum CIKind {
    #[strum(serialize = "gitlabci")]
    GitLab,
}

#[derive(Clone, Debug)]
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
                let merge_request = MergeRequest { number };
                Ok(Self { url, merge_request })
            }
        }
    }

    pub const fn url(&self) -> &String {
        &self.url
    }

    pub const fn merge_request(&self) -> &MergeRequest {
        &self.merge_request
    }
}

#[derive(Clone, Debug)]
pub struct MergeRequest {
    number: u64,
}

impl MergeRequest {
    pub const fn number(&self) -> &u64 {
        &self.number
    }
}
