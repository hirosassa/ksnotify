pub mod gitlab;
use crate::template;

use anyhow::Result;
use strum_macros::EnumString;

#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumString)]
pub enum NotifierKind {
    #[strum(serialize = "gitlab")]
    GitLab,
    #[strum(serialize = "slack")]
    Slack,
}

pub trait Notifiable {
    fn notify(&self, body: template::Template, patch: bool) -> Result<()>;
}
