mod ci;
mod notifier;

use anyhow::{anyhow, Result};
use handlebars::Handlebars;
use notifier::Notifiable;
use serde::Serialize;
use std::fs::File;
use std::io::{self, Read};
use std::process;
use std::str::FromStr;
use std::string::ToString;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use yaml_rust::{Yaml, YamlLoader};

#[derive(Debug, PartialEq, Clone, Copy, Display, EnumIter)]
pub enum NotifierKind {
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

#[derive(Serialize)]
struct Template {
    title: String,
    result: String,
    body: String,
    link: String,
}

impl Template {
    const DEFAULT_BUILD_TITLE: &'static str = "## Build result";

    const DEFAULT_BUILD_TEMPLATE: &'static str = "
{{ title }} <sup>[CI link]( {{ link }} )</sup>
<details><summary>Details (Click me)</summary>
<pre><code> {{ body }}
</pre></code></details>
";

    fn new(body: String, ci: ci::CI) -> Self {
        Self {
            title: Template::DEFAULT_BUILD_TITLE.to_string(),
            result: "".to_string(),
            body,
            link: ci.url().to_string(),
        }
    }

    fn render(&self) -> Result<String> {
        let reg = Handlebars::new();
        let j = serde_json::to_value(self).unwrap();
        Ok(reg.render_template(Template::DEFAULT_BUILD_TEMPLATE, &j)?)
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
        NotifierKind::GitLab => Box::new(notifier::gitlab::GitlabNotifier::new(ci.clone(), config.notifier)?) as Box<dyn Notifiable>,
        NotifierKind::Slack => Box::new(notifier::slack::SlackNotifier::new(config.notifier)?) as Box<dyn Notifiable>,
    };

    let mut body = String::new();
    io::stdin().read_to_string(&mut body)?;

    let template = Template::new(body, ci);

    notifier.notify(template.render()?)?;
    Ok(())
}
