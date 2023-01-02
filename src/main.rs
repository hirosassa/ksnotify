#![allow(clippy::use_self)]
mod ci;
mod config;
mod notifier;
mod parser;
mod template;

use anyhow::{Context, Result};
use clap::Parser;
use clap_verbosity_flag::Verbosity;
use log::{debug, error, info};
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

    /// Update an existing comment instead of creating a new comment. If there is no existing comment, a new comment is created.
    #[arg(long)]
    pub patch: bool,

    /// Target component name to distinguish for each environments or product.
    #[arg(long)]
    pub target: Option<String>,

    /// Whether if suppress diffs comes from Skaffold labels.
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
    debug!("verbose mode");

    let config =
        config::Config::new(&cli).with_context(|| format!("failed to load config: {:?}", cli))?;
    info!("config: {:?}", config);

    // Local PC (for debug)
    if config.ci == ci::CIKind::Local {
        let content = process(&config, None, cli.target)?;
        println!("{}", content.render()?);
        return Ok(());
    }

    let ci =
        ci::CI::new(config.ci).with_context(|| format!("failed to create CI: {:?}", config.ci))?;
    let notifier_kind = config.notifier;
    let notifier = match notifier_kind {
        notifier::NotifierKind::GitLab => notifier::gitlab::GitlabNotifier::new(&ci),
        notifier::NotifierKind::Slack => todo!(),
    }
    .with_context(|| format!("failed to create notifier: {:?}", ci))?;

    let template = process(&config, Some(ci.job_url().to_string()), cli.target)?;
    notifier
        .notify(template, config.patch)
        .with_context(|| "failed to notify".to_string())?;
    Ok(())
}

fn process(
    config: &config::Config,
    url: Option<String>,
    target: Option<String>,
) -> Result<template::Template> {
    let mut body = String::new();
    io::stdin().read_to_string(&mut body)?;
    let parser = parser::DiffParser::new(config.suppress_skaffold)?;
    let result = parser.parse(&body)?;
    let link = url.unwrap_or_default();
    let template = template::Template::new(result.kind_result, link, target);
    Ok(template)
}
