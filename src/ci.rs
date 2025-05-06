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
        info!("create ci with {:?}", ci);
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
