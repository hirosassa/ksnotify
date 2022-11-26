use std::collections::HashMap;

use anyhow::Result;
use handlebars::Handlebars;
use itertools::Itertools;
use serde::Serialize;

#[derive(Serialize)]
pub struct Template {
    target: Option<String>,
    changed_kinds: String,
    details: String,
    link: String,
    is_no_changes: bool,
}

impl Template {
    const DEFAULT_BUILD_TITLE_TEMPLATE: &'static str =
        "## Plan result{{#if target}} ({{target}}){{/if}}";
    const DEFAULT_BUILD_BODY_TEMPLATE: &'static str = "
[CI link]( {{ link }} )

{{#if is_no_changes}}
```
No changes. Kubernetes configurations are up-to-date.
```
{{else}}
* updated
{{ changed_kinds }}

<details><summary>Details (Click me)</summary>

{{{ details }}}

</details>
{{/if}}
";

    pub fn new(results: HashMap<String, String>, link: String, target: Option<String>) -> Self {
        let changed_kinds = Self::generate_changed_kinds_markdown(&results);
        let details = Self::generate_details_markdown(&results);
        let is_no_changes = results.is_empty();
        Self {
            target,
            changed_kinds,
            details,
            link,
            is_no_changes,
        }
    }

    pub fn render(&self) -> Result<String> {
        let reg = Handlebars::new();
        let j = serde_json::to_value(self).unwrap();
        let title = reg.render_template(Self::DEFAULT_BUILD_TITLE_TEMPLATE, &j)?;
        let body = reg.render_template(Self::DEFAULT_BUILD_BODY_TEMPLATE, &j)?;
        Ok(format!("{}{}", title, body))
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
                let title = format!("### {}", k);
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
        let expected = "### test1\n```diff\nABC\n```\n### test2\n```diff\nDEF\n```".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_new_with_no_changes() {
        let results: HashMap<String, String> = HashMap::new();
        let link = "http://example.com".to_string();
        let target = Some("sample app".to_string());
        let t = Template::new(results, link, target);
        assert_eq!(t.is_no_changes, true);
    }

    #[test]
    fn test_new_with_some_changes() {
        let mut results: HashMap<String, String> = HashMap::new();
        results.insert("sample".to_string(), "change content".to_string());
        let link = "http://example.com".to_string();
        let target = Some("sample app".to_string());
        let t = Template::new(results, link, target);
        assert_eq!(t.is_no_changes, false);
    }

    #[test]
    fn test_render_title_with_target() {
        let reg = Handlebars::new();
        let mut data = HashMap::new();
        data.insert("target".to_string(), "sample".to_string());
        let actual = reg
            .render_template(Template::DEFAULT_BUILD_TITLE_TEMPLATE, &data)
            .unwrap();
        let expected = "## Plan result (sample)".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_render_title_without_target() {
        let reg = Handlebars::new();
        let data = HashMap::<String, String>::new();
        let actual = reg
            .render_template(Template::DEFAULT_BUILD_TITLE_TEMPLATE, &data)
            .unwrap();
        let expected = "## Plan result".to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_render_body_with_no_changes() {
        let reg = Handlebars::new();
        let mut data = HashMap::new();
        data.insert("is_no_changes".to_string(), "true".to_string());
        data.insert("link".to_string(), "http://example.com".to_string());
        let actual = reg
            .render_template(Template::DEFAULT_BUILD_BODY_TEMPLATE, &data)
            .unwrap();
        let expected = "
[CI link]( http://example.com )

```
No changes. Kubernetes configurations are up-to-date.
```
"
        .to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_render_body_with_changes() {
        let reg = Handlebars::new();
        let mut data = HashMap::new();
        data.insert("link".to_string(), "http://example.com".to_string());
        data.insert("link".to_string(), "http://example.com".to_string());
        let actual = reg
            .render_template(Template::DEFAULT_BUILD_BODY_TEMPLATE, &data)
            .unwrap();
        let expected = "
[CI link]( http://example.com )

* updated


<details><summary>Details (Click me)</summary>



</details>
"
        .to_string();
        assert_eq!(actual, expected);
    }
}
