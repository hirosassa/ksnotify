use std::collections::HashMap;

use anyhow::Result;
use handlebars::Handlebars;
use itertools::Itertools;
use serde::Serialize;

#[derive(Serialize)]
pub struct Template {
    title: String,
    changed_kinds: String,
    details: String,
    link: String,
}

impl Template {
    const DEFAULT_BUILD_TITLE: &'static str = "## Plan result";

    const DEFAULT_BUILD_TEMPLATE: &'static str = "{{ title }}

[CI link]( {{ link }} )

* updated
{{ changed_kinds }}

<details><summary>Details (Click me)</summary>

{{{ details }}}

</details>
";

    pub fn new(results: HashMap<String, String>, link: String) -> Self {
        let changed_kinds = Self::generate_changed_kinds_markdown(&results);
        let details = Self::generate_details_markdown(&results);
        Self {
            title: Self::DEFAULT_BUILD_TITLE.to_string(),
            changed_kinds,
            details,
            link,
        }
    }

    pub fn render(&self) -> Result<String> {
        let reg = Handlebars::new();
        let j = serde_json::to_value(self).unwrap();
        Ok(reg.render_template(Self::DEFAULT_BUILD_TEMPLATE, &j)?)
    }

    fn generate_changed_kinds_markdown(results: &HashMap<String, String>) -> String {
        let kinds: Vec<String> = results.keys().map(|e| e.to_string()).sorted().collect();
        format!("  * {}", kinds.join("\n  * "))
    }

    fn generate_details_markdown(results: &HashMap<String, String>) -> String {
        let kinds: Vec<String> = results.keys().map(|e| e.to_string()).sorted().collect();
        let details: Vec<String> = kinds
            .iter()
            .map(|k| {
                let title = format!("## {}", k);
                let body = format!("```diff\n{}\n```", results[k]);
                format!("{}\n{}", title, body)
            })
            .collect();
        details.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_changed_kinds_markdown() {
        let kinds = vec!["test1", "test2"];
        let details = vec!["ABC", "DEF"];
        let data: HashMap<_, _> = kinds
            .iter()
            .zip(details.iter())
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let actual = Template::generate_changed_kinds_markdown(&data);
        let expected = "  * test1\n  * test2".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_generate_details_markdown() {
        let kinds = vec!["test1", "test2"];
        let details = vec!["ABC", "DEF"];
        let data: HashMap<_, _> = kinds
            .iter()
            .zip(details.iter())
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let actual = Template::generate_details_markdown(&data);
        let expected = "## test1\n```diff\nABC\n```\n## test2\n```diff\nDEF\n```".to_string();
        assert_eq!(actual, expected);
    }
}
