use crate::ci;
use crate::Cli;

use anyhow::Result;
use log::info;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub ci: ci::CIKind,
    pub suppress_skaffold: bool,
    pub ignore_tag_images: Vec<String>,
    pub patch: bool,
}

impl Config {
    pub fn new(cli: &Cli) -> Result<Self> {
        info!("load config");

        // todo: validate cli args
        if let Some(ci_kind) = cli.ci.as_deref() {
            let ci = ci::CIKind::from_str(ci_kind)?;
            let suppress_skaffold = cli.suppress_skaffold;
            let ignore_tag_images = cli.ignore_tag_images.clone();
            let patch = cli.patch;
            return Ok(Self {
                ci,
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
        let suppress_skaffold = env::var("KSNOTIFY_SUPPRESS_SKAFFOLD").is_ok();
        let ignore_tag_images = env::var("KSNOTIFY_IGNORE_TAG_IMAGES")?
            .split(',')
            .map(String::from)
            .collect();
        let patch = env::var("KSNOTIFY_PATCH").is_ok();
        Ok(Self {
            ci,
            suppress_skaffold,
            ignore_tag_images,
            patch,
        })
    }
}

#[cfg(test)]
mod tests {
    use clap_verbosity_flag::{ErrorLevel, Verbosity};

    use super::*;

    #[test]
    fn test_new_from_env() {
        temp_env::with_vars(
            [
                ("KSNOTIFY_CI", Some("github")),
                ("KSNOTIFY_SUPPRESS_SKAFFOLD", Some("true")),
                ("KSNOTIFY_IGNORE_TAG_IMAGES", Some("image1,image2")),
                ("KSNOTIFY_PATCH", Some("true")),
            ],
            || {
                let config = Config::new(&Cli {
                    ci: None,
                    target: None,
                    suppress_skaffold: false,
                    ignore_tag_images: vec![],
                    patch: false,
                    config: None,
                    verbose: Verbosity::<ErrorLevel>::default(),
                })
                .unwrap();

                assert_eq!(config.ci, ci::CIKind::GitHub);
                assert!(config.suppress_skaffold);
                assert_eq!(config.ignore_tag_images, vec!["image1", "image2"]);
                assert!(config.patch);
            },
        );
    }

    #[test]
    fn test_new_from_file() {
        let config_content = r#"
ci: gitlab
suppress_skaffold: false
ignore_tag_images:
  - image1
  - image2
patch: false
"#;

        let temp_dir = tempdir::TempDir::new("temp").unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(&config_path, config_content).unwrap();

        let config = Config::new(&Cli {
            ci: None,
            target: None,
            suppress_skaffold: false,
            ignore_tag_images: vec![],
            patch: false,
            config: Some(config_path),
            verbose: Verbosity::<ErrorLevel>::default(),
        })
        .unwrap();

        assert_eq!(config.ci, ci::CIKind::GitLab);
        assert!(!config.suppress_skaffold);
        assert_eq!(config.ignore_tag_images, vec!["image1", "image2"]);
        assert!(!config.patch);
    }

    #[test]
    fn test_new_with_cli_args() {
        let config = Config::new(&Cli {
            ci: Some("github".to_string()),
            target: None,
            suppress_skaffold: true,
            ignore_tag_images: vec!["image1".to_string(), "image2".to_string()],
            patch: true,
            config: None,
            verbose: Verbosity::<ErrorLevel>::default(),
        })
        .unwrap();

        assert_eq!(config.ci, ci::CIKind::GitHub);
        assert!(config.suppress_skaffold);
        assert_eq!(config.ignore_tag_images, vec!["image1", "image2"]);
        assert!(config.patch);
    }
}
