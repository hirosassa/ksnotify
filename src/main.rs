use anyhow::Result;
use gitlab::api::{self, projects::merge_requests::notes::CreateMergeRequestNote, Query};
use gitlab::Gitlab;
use handlebars::Handlebars;
use serde::Serialize;
use serde_json;
use std::fs::File;
use std::io::{self, Read};
use std::{env, process};
use yaml_rust::YamlLoader;

#[derive(Debug)]
struct GitlabConfig {
    base_url: String,
    token: String,
    repository: Repository,
}

#[derive(Debug)]
struct Repository {
    owner: String,
    project: String,
}

struct MergeRequest {
    number: u64,
    revision: String,
}

struct CI {
    url: String,
    merge_request: MergeRequest,
}

impl CI {
    fn new() -> Result<CI> {
        let url = env::var("CI_JOB_URL")?;
        let number = env::var("CI_MERGE_REQUEST_IID")?.parse()?;
        let revision = env::var("CI_COMMIT_SHA")?;
        let merge_request = MergeRequest { number, revision };
        Ok(CI { url, merge_request })
    }
}

struct Notifier {
    client: Client,
    config: Config,
    ci: CI,
}

impl Notifier {
    fn new(ci: CI, path: &str) -> Result<Self> {
        let config = Config::new(path)?;
        let gitlab = Gitlab::new(
            config.gitlab_config.base_url.to_owned(),
            config.gitlab_config.token.to_owned(),
        )?;
        let client = Client { client: gitlab };
        Ok(Self { client, config, ci })
    }
}

trait Notifiable {
    fn notify(&self, body: String) -> Result<()>;
}

impl Notifiable for Notifier {
    fn notify(&self, body: String) -> Result<()> {
        let project = format!(
            "{}/{}",
            self.config.gitlab_config.repository.owner,
            self.config.gitlab_config.repository.project
        );
        let note = CreateMergeRequestNote::builder()
            .project(project)
            .merge_request(self.ci.merge_request.number)
            .body(body)
            .build()
            .map_err(anyhow::Error::msg)?;
        api::ignore(note).query(&self.client.client)?;
        Ok(())
    }
}

struct Client {
    client: Gitlab,
}

#[derive(Debug)]
struct Config {
    ci: String,
    gitlab_config: GitlabConfig,
}

impl Config {
    fn new(path: &str) -> Result<Self> {
        let mut f = File::open(path)?;
        let mut config_string = String::new();
        f.read_to_string(&mut config_string)?;
        let docs = YamlLoader::load_from_str(&config_string).unwrap();

        Ok(Self {
            ci: docs[0]["ci"].as_str().unwrap().to_string(),
            gitlab_config: GitlabConfig {
                base_url: docs[0]["gitlab"]["base_url"].as_str().unwrap().to_string(),
                token: docs[0]["gitlab"]["token"].as_str().unwrap().to_string(),
                repository: Repository {
                    owner: docs[0]["gitlab"]["repository"]["owner"]
                        .as_str()
                        .unwrap()
                        .to_string(),
                    project: docs[0]["gitlab"]["repository"]["project"]
                        .as_str()
                        .unwrap()
                        .to_string(),
                },
            },
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
    let ci = CI::new()?;
    let notifier = Notifier::new(ci, "ksnotify.yaml")?;

    let mut body = String::new();
    io::stdin().read_to_string(&mut body)?;

    let template = Template::new(body);

    notifier.notify(template.render()?)?;
    Ok(())
}
