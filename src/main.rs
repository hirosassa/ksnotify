use gitlab::api::{
    self, projects::merge_requests::discussions::CreateMergeRequestDiscussion, Query,
};
use gitlab::Gitlab;
use std::env;
use std::fs::File;
use std::io::{self, Read};
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
    fn new() -> CI {
        let mr = MergeRequest {
            number: env::var("CI_MERGE_REQUEST_IID")
                .expect("CI_MERGE_REQUEST_IID is not set")
                .parse()
                .unwrap(),
            revision: env::var("CI_COMMIT_SHA").expect("CI_COMMIT_SHA is not set"),
        };
        CI {
            url: env::var("CI_JOB_URL").expect("CI_JOB_URL is not set"),
            merge_request: mr,
        }
    }
}

struct Notifier {
    client: Client,
    config: Config,
    ci: CI,
}

impl Notifier {
    fn new(ci: CI, path: &str) -> Notifier {
        let config = Config::new(path);
        println!("confing: {:?}", config);
        Notifier {
            client: Client {
                client: Gitlab::new(
                    Config::new(path).gitlab_config.base_url,
                    Config::new(path).gitlab_config.token,
                )
                .unwrap(),
            },
            config: Config::new(path),
            ci: ci,
        }
    }
}

trait Notifiable {
    fn notify(&self, body: String);
}
impl Notifiable for Notifier {
    fn notify(&self, body: String) {
        let discussion = CreateMergeRequestDiscussion::builder()
            .project(self.config.gitlab_config.repository.project.as_ref())
            .merge_request(self.ci.merge_request.number)
            .body(body)
            .build()
            .unwrap();
        let _ = api::ignore(discussion).query(&self.client.client).unwrap();
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
    fn new(path: &str) -> Self {
        let mut f = File::open(path).expect("file not found");
        let mut config_string = String::new();
        f.read_to_string(&mut config_string)
            .expect("failed to load config");
        let docs = YamlLoader::load_from_str(&config_string).unwrap();

        Self {
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
        }
    }
}

fn main() {
    let ci = CI::new();
    let notifier = Notifier::new(ci, "./ksnotify.yaml");

    let mut body = String::new();
    io::stdin().read_line(&mut body).expect("failed to read");
    println!("{}", body);

    notifier.notify(body);
}
