#![allow(clippy::use_self)]
mod ci;
mod config;
mod notifier;
mod parser;
mod template;

use anyhow::{Context, Result};
use clap::Parser;
use clap_verbosity_flag::Verbosity;
use log::{error, info};
use notifier::Notifiable;
use parser::Parsable;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process;
use std::string::ToString;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// CI platform name.
    #[arg(long)]
    pub ci: Option<String>,

    /// Target platform to notify.
    #[arg(long)]
    pub notifier: Option<String>,

    /// Whether if suppress diffs comes from Skaffold labels
    #[arg(long)]
    pub suppress_skaffold: bool,

    /// Path of config file in YAML format. This option cannot conjunction with ci and notifier options.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[clap(flatten)]
    verbose: Verbosity,
}

fn main() {
    if let Err(err) = run() {
        error!("Error: {:#?}", err);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    let config =
        config::Config::new(&cli).with_context(|| format!("failed to load config: {:?}", cli))?;
    info!("config: {:?}", config);
    let ci =
        ci::CI::new(config.ci).with_context(|| format!("failed to create CI: {:?}", config.ci))?;
    let notifier_kind = config.notifier;
    let notifier = match notifier_kind {
        notifier::NotifierKind::GitLab => notifier::gitlab::GitlabNotifier::new(&ci),
        notifier::NotifierKind::Slack => todo!(),
    }
    .with_context(|| format!("failed to create notifier: {:?}", ci))?;

    let mut body = String::new();
    io::stdin().read_to_string(&mut body)?;
    let parser = parser::DiffParser::new(config.suppress_skaffold)?;
    let result = parser.parse(&body)?;
    let template = template::Template::new(result.kind_result, ci.job_url().to_string());

    notifier
        .notify(template.render()?)
        .with_context(|| format!("failed to notify"))?;
    Ok(())
}
