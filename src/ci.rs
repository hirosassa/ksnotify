mod notifier;

use anyhow::{Context, Result};
use log::info;
use notifier::Notifiable;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumString, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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

pub struct CI {
    pub notifier: Box<dyn Notifiable>,
}

impl CI {
    pub fn new(ci: CIKind) -> Result<Self> {
        info!("create ci with {ci:?}");
        match ci {
            CIKind::GitLab => {
                let notifier: Box<dyn Notifiable> = Box::new(
                    notifier::gitlab::GitlabNotifier::new()
                        .with_context(|| "failed to create GitLab notifier")?,
                );
                Ok(Self { notifier })
            }
            CIKind::GitHub => {
                let notifier: Box<dyn Notifiable> = Box::new(
                    notifier::github::GithubNotifier::new()
                        .with_context(|| "failed to create GitHub notifier")?,
                );

                Ok(Self { notifier })
            }
            CIKind::Local => todo!(), // do nothing if local run. never reach here.
        }
    }

    pub fn job_url(&self) -> String {
        self.notifier.job_url()
    }
}

#[derive(Clone, Debug)]
pub struct MergeRequest {
    pub number: Option<u64>,
    pub commit_sha: String,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_ci_kind_from_str_github() {
        let actual = CIKind::from_str("github").unwrap();
        assert_eq!(actual, CIKind::GitHub);
    }

    #[test]
    fn test_ci_kind_from_str_gitlab() {
        let actual = CIKind::from_str("gitlab").unwrap();
        assert_eq!(actual, CIKind::GitLab);
    }

    #[test]
    fn test_ci_kind_from_str_local() {
        let actual = CIKind::from_str("local").unwrap();
        assert_eq!(actual, CIKind::Local);
    }

    #[test]
    fn test_ci_kind_from_str_invalid() {
        let actual = CIKind::from_str("invalid");
        assert!(actual.is_err());
    }
}
