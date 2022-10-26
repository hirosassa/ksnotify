use crate::Cli;
use crate::{ci, notifier};

use anyhow::Result;
use log::info;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use strum_macros::{Display, EnumIter};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Display, EnumIter)]
enum NotifierKind {
    #[strum(serialize = "gitlab")]
    GitLab,
    #[strum(serialize = "slack")]
    Slack,
}

#[derive(Debug)]
pub struct Config {
    pub ci: ci::CIKind,
    pub notifier: notifier::NotifierKind,
    pub suppress_skaffold: bool,
}

impl Config {
    pub fn new(cli: &Cli) -> Result<Self> {
        info!("load config");

        // todo: validate cli args
        if let (Some(ci_kind), Some(notifier_kind)) = (cli.ci.as_deref(), cli.notifier.as_deref()) {
            let ci = ci::CIKind::from_str(ci_kind)?;
            let notifier = notifier::NotifierKind::from_str(notifier_kind)?;
            let suppress_skaffold = cli.suppress_skaffold;
            return Ok(Self {
                ci,
                notifier,
                suppress_skaffold,
            });
        }

        cli.config
            .as_deref()
            .map_or_else(Self::from_env, |path| Self::from_file(path.to_path_buf()))
    }

    fn from_file(path: PathBuf) -> Result<Self> {
        info!("cli arguments are not set, use configuration file");
        let config_string = fs::read_to_string(path)?;
        let doc: HashMap<String, String> = serde_yaml::from_str(&config_string)?;
        let ci = ci::CIKind::from_str(doc.get("ci").expect("failed to load the CI type"))?;
        let notifier = notifier::NotifierKind::from_str(
            doc.get("notifier")
                .expect("failed to load the Notifier type"),
        )?;
        let suppress_skaffold = doc
            .get("suppress_skaffold")
            .expect("failed to load the suppress_skaffold flag")
            .parse::<bool>()?;

        Ok(Self {
            ci,
            notifier,
            suppress_skaffold,
        })
    }

    fn from_env() -> Result<Self> {
        info!("config file is not found, use environmental variables");
        let ci = ci::CIKind::from_str(&env::var("KSNOTIFY_CI")?)?;
        let notifier = notifier::NotifierKind::from_str(&env::var("KSNOTIFY_NOTIFIER")?)?;
        let suppress_skaffold = matches!(env::var("KSNOTIFY_SUPPRESS_SKAFFOLD"), Ok(_));
        Ok(Self {
            ci,
            notifier,
            suppress_skaffold,
        })
    }
}
