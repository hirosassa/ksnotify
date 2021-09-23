mod notifier;

use anyhow::Result;
use handlebars::Handlebars;
use notifier::Notifiable;
use serde::Serialize;
use std::fs::File;
use std::io::{self, Read};
use std::str::FromStr;
use std::{env, process};
use strum_macros::EnumString;
use yaml_rust::{Yaml, YamlLoader};

#[derive(Debug, PartialEq, Clone, Copy, EnumString)]
pub enum CIKind {
    #[strum(serialize = "gitlab")]
    GitLab,
}

#[derive(Debug)]
struct MergeRequest {
    number: u64,
    revision: String,
}

#[derive(Debug)]
pub struct CI {
    url: String,
    merge_request: MergeRequest,
}

impl CI {
    fn new(ci: CIKind) -> Result<Self> {
        match ci {
            CIKind::GitLab => {
                // todo: make this as function
                let url = env::var("CI_JOB_URL")?;
                let number = env::var("CI_MERGE_REQUEST_IID")?.parse()?;
                let revision = env::var("CI_COMMIT_SHA")?;
                let merge_request = MergeRequest { number, revision };
                Ok(Self { url, merge_request })
            }
        }
    }
}

#[derive(Debug)]
struct Config {
    ci: CIKind,
    notifier: Yaml,
}

impl Config {
    fn new(path: &str) -> Result<Self> {
        let mut f = File::open(path)?;
        let mut config_string = String::new();
        f.read_to_string(&mut config_string)?;
        let doc = &YamlLoader::load_from_str(&config_string)?[0];
        let ci_kind = CIKind::from_str(doc["ci"].as_str().expect("failed to load the CI type"))?;

        Ok(Self {
            ci: ci_kind,
            notifier: doc["notifier"].clone(),
        })
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

    fn new(body: String) -> Self {
        Self {
            title: Template::DEFAULT_BUILD_TITLE.to_string(),
            result: "".to_string(),
            body,
            link: "".to_string(),
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
    let ci = CI::new(config.ci)?;

    let notifier = match config.ci {
        CIKind::GitLab => notifier::gitlab::GitlabNotifier::new(ci, config.notifier),
    }?;

    let mut body = String::new();
    io::stdin().read_to_string(&mut body)?;

    let template = Template::new(body);

    notifier.notify(template.render()?)?;
    Ok(())
}
