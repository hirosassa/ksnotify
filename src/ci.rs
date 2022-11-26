use anyhow::{Context, Result};
use log::info;
use std::env;
use strum_macros::EnumString;

#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumString)]
pub enum CIKind {
    /// ksnotify is running on GitLab CI.
    #[strum(serialize = "gitlab")]
    GitLab,

    /// ksnotify is running on Local PC (for debug).
    #[strum(serialize = "local")]
    Local,
}

#[derive(Clone, Debug)]
pub struct CI {
    job_url: String,
    merge_request: MergeRequest,
}

impl CI {
    pub fn new(ci: CIKind) -> Result<Self> {
        info!("create ci with {:?}", ci);
        match ci {
            CIKind::GitLab => {
                // todo: make this as function
                let job_url = env::var("CI_JOB_URL")
                    .with_context(|| "CI_JOB_URL is not provided.".to_string())?;
                let number = env::var("CI_MERGE_REQUEST_IID")
                    .with_context(|| "CI_MERGE_REQUEST_IID is not provided".to_string())?
                    .parse()?;
                let merge_request = MergeRequest { number };
                Ok(Self {
                    job_url,
                    merge_request,
                })
            }
            CIKind::Local => Ok(Self {
                job_url: "".to_string(),
                merge_request: MergeRequest { number: 1 },
            }),
        }
    }

    pub const fn job_url(&self) -> &String {
        &self.job_url
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
