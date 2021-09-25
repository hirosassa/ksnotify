pub mod gitlab;
use anyhow::Result;

pub trait Notifiable {
    fn notify(&self, body: String) -> Result<()>;
}
