#![allow(clippy::use_self)]

mod ci;
mod notifier;
mod parser;
mod template;

use anyhow::{anyhow, Result};
use notifier::Notifiable;
use parser::Parsable;
use std::fs::File;
use std::io::{self, Read};
use std::process;
use std::str::FromStr;
use std::string::ToString;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use yaml_rust::{Yaml, YamlLoader};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Display, EnumIter)]
enum NotifierKind {
    #[strum(serialize = "gitlab")]
    GitLab,
    #[strum(serialize = "slack")]
    Slack,
}

#[derive(Debug)]
struct Config {
    ci: ci::CIKind,
    notifier: Yaml,
}

impl Config {
    fn new(path: &str) -> Result<Self> {
        let mut f = File::open(path)?;
        let mut config_string = String::new();
        f.read_to_string(&mut config_string)?;
        let doc = &YamlLoader::load_from_str(&config_string)?[0];
        let ci_kind =
            ci::CIKind::from_str(doc["ci"].as_str().expect("failed to load the CI type"))?;

        Ok(Self {
            ci: ci_kind,
            notifier: doc["notifier"].clone(),
        })
    }

    fn select_notifier(&self) -> Result<NotifierKind> {
        if let Some(hash) = self.notifier.as_hash() {
            for kind in NotifierKind::iter() {
                if hash.contains_key(&Yaml::String(kind.to_string())) {
                    return Ok(kind);
                }
            }
        }
        Err(anyhow!("invalid notifier type"))
    }
}

fn main() {
    let result = run();

    match result {
        Ok(_) => process::exit(0),
        Err(e) => {
            eprintln!("ksnotify: {}", e);
            process::exit(1);
        }
    }
}

fn run() -> Result<()> {
    let config = Config::new("ksnotify.yaml")?;
    let ci = ci::CI::new(config.ci)?;

    let notifier_kind = config.select_notifier()?;
    let notifier = match notifier_kind {
        NotifierKind::GitLab => notifier::gitlab::GitlabNotifier::new(ci.clone(), config.notifier),
        NotifierKind::Slack => todo!(),
    }?;

    let mut body = String::new();
    io::stdin().read_to_string(&mut body)?;
    let parser = parser::DiffParser::new()?;
    let result = parser.parse(&body)?;
    let template = template::Template::new(result.kind_result, ci.url().to_string());

    notifier.notify(template.render()?)?;
    Ok(())
}
