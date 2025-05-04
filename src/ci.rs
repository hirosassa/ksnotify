use anyhow::{Context, Result};
use log::info;
use serde::{Deserialize, Serialize};
use std::env;
use strum_macros::EnumString;

#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumString, Serialize, Deserialize)]
pub enum CIKind {
    /// ksnotify is running on GitLab CI.
    #[strum(serialize = "gitlab")]
    GitLab,

    /// ksnotify is running on GitHub Actions.
    #[strum(serialize = "github")]
    GitHub,

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

                // CI_MERGE_REQUEST_IID is optional, but should be u64.
                let number = env::var("CI_MERGE_REQUEST_IID").ok();
                let number = if number.is_some() {
                    Some(number.unwrap().parse::<u64>()?)
                } else {
                    None
                };

                let commit_sha = env::var("CI_COMMIT_SHA")
                    .with_context(|| "CI_COMMIT_SHA is not provided.".to_string())?;

                let merge_request = MergeRequest { number, commit_sha };
                Ok(Self {
                    job_url,
                    merge_request,
                })
            }
            CIKind::GitHub => {
                let repository = env::var("GITHUB_REPOSITORY")
                    .with_context(|| "GITHUB_REPOSITORY is not provided.".to_string())?;
                let run_id = env::var("GITHUB_RUN_ID")
                    .with_context(|| "GITHUB_RUN_ID is not provided.".to_string())?;
                let number = env::var("GITHUB_REF_NAME").ok();
                let number = if number.is_some() {
                    Some(number.unwrap().split("/").next().unwrap().parse::<u64>()?)
                } else {
                    None
                };
                let commit_sha = env::var("GITHUB_SHA")
                    .with_context(|| "GITHUB_SHA is not provided.".to_string())?;

                let job_url = format!("https://github.com/{}/actions/runs/{}", repository, run_id);
                let merge_request = MergeRequest { number, commit_sha };
                Ok(Self {
                    job_url,
                    merge_request,
                })
            }
            CIKind::Local => Ok(Self {
                job_url: "".to_string(),
                merge_request: MergeRequest {
                    number: None,
                    commit_sha: "".to_string(),
                },
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
    pub number: Option<u64>,
    pub commit_sha: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ci_new_gitlab() {
        temp_env::with_vars(
            [
                ("CI_JOB_URL", Some("https://gitlab.com/ksnotify")),
                ("CI_MERGE_REQUEST_IID", Some("123")),
                ("CI_COMMIT_SHA", Some("abcdefg")),
            ],
            || {
                let ci = CI::new(CIKind::GitLab).unwrap();
                assert_eq!(ci.job_url(), "https://gitlab.com/ksnotify");
                assert_eq!(ci.merge_request().number, Some(123));
                assert_eq!(ci.merge_request().commit_sha, "abcdefg");
            },
        );
    }

    #[test]
    fn test_ci_new_gitlab_without_merge_request() {
        temp_env::with_vars(
            [
                ("CI_JOB_URL", Some("https://gitlab.com/ksnotify")),
                ("CI_COMMIT_SHA", Some("abcdefg")),
            ],
            || {
                let ci = CI::new(CIKind::GitLab).unwrap();
                assert_eq!(ci.job_url(), "https://gitlab.com/ksnotify");
                assert_eq!(ci.merge_request().number, None);
                assert_eq!(ci.merge_request().commit_sha, "abcdefg");
            },
        );
    }

    #[test]
    fn test_ci_new_github() {
        temp_env::with_vars(
            [
                ("GITHUB_REPOSITORY", Some("hirosassa/ksnotify")),
                ("GITHUB_RUN_ID", Some("123")),
                ("GITHUB_REF_NAME", Some("123/merge")),
                ("GITHUB_SHA", Some("abcdefg")),
            ],
            || {
                let ci = CI::new(CIKind::GitHub).unwrap();
                assert_eq!(
                    ci.job_url(),
                    "https://github.com/hirosassa/ksnotify/actions/runs/123"
                );
                assert_eq!(ci.merge_request().number, Some(123));
                assert_eq!(ci.merge_request().commit_sha, "abcdefg");
            },
        );
    }

    #[test]
    fn test_ci_new_without_ci() {
        let ci = CI::new(CIKind::Local).unwrap();
        assert_eq!(ci.job_url(), "");
        assert_eq!(ci.merge_request().number, None);
        assert_eq!(ci.merge_request().commit_sha, "");
    }
}
