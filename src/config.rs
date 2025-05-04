use crate::Cli;
use crate::{ci, notifier};

use anyhow::Result;
use log::info;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use strum_macros::{Display, EnumIter};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Display, EnumIter)]
enum NotifierKind {
    #[strum(serialize = "gitlab")]
    GitLab,
    #[strum(serialize = "github")]
    GitHub,
    #[strum(serialize = "slack")]
    Slack,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub ci: ci::CIKind,
    pub notifier: notifier::NotifierKind,
    pub suppress_skaffold: bool,
    pub ignore_tag_images: Vec<String>,
    pub patch: bool,
}

impl Config {
    pub fn new(cli: &Cli) -> Result<Self> {
        info!("load config");

        // todo: validate cli args
        if let (Some(ci_kind), Some(notifier_kind)) = (cli.ci.as_deref(), cli.notifier.as_deref()) {
            let ci = ci::CIKind::from_str(ci_kind)?;
            let notifier = notifier::NotifierKind::from_str(notifier_kind)?;
            let suppress_skaffold = cli.suppress_skaffold;
            let ignore_tag_images = cli.ignore_tag_images.clone();
            let patch = cli.patch;
            return Ok(Self {
                ci,
                notifier,
                suppress_skaffold,
                ignore_tag_images,
                patch,
            });
        }

        cli.config
            .as_deref()
            .map_or_else(Self::from_env, |path| Self::from_file(path.to_path_buf()))
    }

    fn from_file(path: PathBuf) -> Result<Self> {
        info!("cli arguments are not set, use configuration file");
        let config_string = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&config_string)?;

        Ok(config)
    }

    fn from_env() -> Result<Self> {
        info!("config file is not found, use environmental variables");
        let ci = ci::CIKind::from_str(&env::var("KSNOTIFY_CI")?)?;
        let notifier = notifier::NotifierKind::from_str(&env::var("KSNOTIFY_NOTIFIER")?)?;
        let suppress_skaffold = env::var("KSNOTIFY_SUPPRESS_SKAFFOLD").is_ok();
        let ignore_tag_images = env::var("KSNOTIFY_IGNORE_TAG_IMAGES")?
            .split(',')
            .map(String::from)
            .collect();
        let patch = env::var("KSNOTIFY_PATCH").is_ok();
        Ok(Self {
            ci,
            notifier,
            suppress_skaffold,
            ignore_tag_images,
            patch,
        })
    }
}
