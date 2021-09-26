pub mod gitlab;
pub mod slack;
use anyhow::Result;

pub trait Notifiable {
    fn notify(&self, body: String) -> Result<()>;
}
