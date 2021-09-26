use anyhow::Result;
use slack_api::sync::chat::{post_message, PostMessageRequest};
use yaml_rust::Yaml;

use super::Notifiable;

pub struct SlackNotifier {
    token: String,
    channel: String,
}

impl SlackNotifier {
    pub fn new(config: Yaml) -> Result<Self> {
        let slack_config = &config["slack"];
        let token = slack_config["token"]
            .as_str()
            .expect("failed to load token of Slack")
            .to_string();
        let channel = slack_config["channel"]
            .as_str()
            .expect("failed to load channel to post")
            .to_string();
        Ok(SlackNotifier { token, channel })
    }
}

impl Notifiable for SlackNotifier {
    fn notify(&self, body: String) -> Result<()> {
        let request = PostMessageRequest {
            channel: self.channel.as_str(),
            text: &body.as_str(),
            ..Default::default()
        };
        let client = slack_api::sync::default_client().expect("Could not get client");
        match post_message(&client, self.token.as_str(), &request) {
            Ok(response) => println!("{:?}", response),
            Err(e) => println!("{:?}", e),
        }
        Ok(())
    }
}
