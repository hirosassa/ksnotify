pub mod github;
pub mod gitlab;
use crate::template;

use anyhow::Result;

pub trait Notifiable {
    fn notify(&self, body: &template::Template, patch: bool) -> Result<()>;
    fn job_url(&self) -> String;
}
