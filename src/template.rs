use std::collections::HashMap;

use anyhow::Result;
use handlebars::Handlebars;
use itertools::Itertools;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Template {
    target: Option<String>,
    configured_kinds: Vec<String>,
    created_kinds: Vec<String>,
    pruned_kinds: Vec<String>,
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
{{#if (gt (len created_kinds) 0)}}
## created
{{#each created_kinds}}
* {{this}}
{{/each}}
{{/if}}
{{#if (gt (len pruned_kinds) 0)}}
## pruned
{{#each pruned_kinds}}
* {{this}}
{{/each}}
{{/if}}
{{#if (gt (len configured_kinds) 0)}}
## configured
{{#each configured_kinds}}
* {{this}}
{{/each}}
{{/if}}

<details><summary>Details (Click me)</summary>

{{{ details }}}

</details>
{{/if}}
";

    pub fn new(results: HashMap<String, String>, link: String, target: Option<String>) -> Self {
        let configured_kinds = Self::generate_configured_kinds_markdown(&results);
        let created_kinds = Self::generate_created_kinds_markdown(&results);
        let pruned_kinds = Self::generate_pruned_kinds_markdown(&results);
        let details = Self::generate_details_markdown(&results);
        let is_no_changes = results.is_empty();
        Self {
            target,
            configured_kinds,
            created_kinds,
            pruned_kinds,
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

    pub fn is_same_build(&self, rendered_string: &str) -> Result<bool> {
        if self.target.is_none() {
            return Ok(false);
        }

        let old_title = match rendered_string.lines().next() {
            // take first line (it should be title)
            Some(title) => title,
            None => return Ok(false),
        };

        let reg = Handlebars::new();
        let j = serde_json::to_value(self).unwrap();
        let current_title = reg.render_template(Self::DEFAULT_BUILD_TITLE_TEMPLATE, &j)?;

        if current_title == old_title {
            return Ok(true);
        }

        Ok(false)
    }

    fn generate_configured_kinds_markdown(results: &HashMap<String, String>) -> Vec<String> {
        let kinds: Vec<String> = results
            .iter()
            .filter(|(_, e)| !e.contains("-kind: "))
            .filter(|(_, e)| !e.contains("+kind: "))
            .map(|(k, _)| k.clone())
            .sorted()
            .collect();
        kinds
    }

    fn generate_created_kinds_markdown(results: &HashMap<String, String>) -> Vec<String> {
        let kinds: Vec<String> = results
            .iter()
            .filter(|(_, e)| e.contains("+kind: "))
            .map(|(k, _)| k.clone())
            .sorted()
            .collect();
        kinds
    }

    fn generate_pruned_kinds_markdown(results: &HashMap<String, String>) -> Vec<String> {
        let kinds: Vec<String> = results
            .iter()
            .filter(|(_, e)| e.contains("-kind: ")) // like "-kind: Deployment"
            .map(|(k, _)| k.clone())
            .sorted()
            .collect();
        kinds
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
    fn test_render_for_created_kinds() {
        let data = HashMap::from([(
            "apps.v1.Deployment.default.hoge".to_string(),
            "+apiVersion: apps/v1
+kind: Deployment
+metadata:
+  name: hoge"
                .to_string(),
        )]);
        let template = Template::new(
            data,
            "https://example.com".to_string(),
            Some("target".to_string()),
        );
        let actual = template.render().unwrap();
        let expected = "## Plan result (target)
[CI link]( https://example.com )

## created
* apps.v1.Deployment.default.hoge

<details><summary>Details (Click me)</summary>

### apps.v1.Deployment.default.hoge
```diff
+apiVersion: apps/v1
+kind: Deployment
+metadata:
+  name: hoge
```

</details>
"
        .to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_render_for_pruned_kinds() {
        let data = HashMap::from([(
            "apps.v1.Deployment.default.hoge".to_string(),
            "-apiVersion: apps/v1
-kind: Deployment
-metadata:
-  name: hoge"
                .to_string(),
        )]);
        let template = Template::new(
            data,
            "https://example.com".to_string(),
            Some("target".to_string()),
        );
        let actual = template.render().unwrap();
        let expected = "## Plan result (target)
[CI link]( https://example.com )

## pruned
* apps.v1.Deployment.default.hoge

<details><summary>Details (Click me)</summary>

### apps.v1.Deployment.default.hoge
```diff
-apiVersion: apps/v1
-kind: Deployment
-metadata:
-  name: hoge
```

</details>
"
        .to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_render_for_configured_kinds() {
        let data = HashMap::from([(
            "apps.v1.Deployment.default.hoge".to_string(),
            "-  image: hoge
+  image: fuga"
                .to_string(),
        )]);
        let template = Template::new(
            data,
            "https://example.com".to_string(),
            Some("target".to_string()),
        );
        let actual = template.render().unwrap();
        let expected = "## Plan result (target)
[CI link]( https://example.com )

## configured
* apps.v1.Deployment.default.hoge

<details><summary>Details (Click me)</summary>

### apps.v1.Deployment.default.hoge
```diff
-  image: hoge
+  image: fuga
```

</details>
"
        .to_string();
        assert_eq!(actual, expected);
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
        let data: HashMap<String, String> = HashMap::new();
        let template = Template::new(
            data,
            "https://example.com".to_string(),
            Some("target".to_string()),
        );
        let actual = template.render().unwrap();
        let expected = "## Plan result (target)
[CI link]( https://example.com )

```
No changes. Kubernetes configurations are up-to-date.
```
"
        .to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_is_same_build_with_target_none() {
        let template = Template {
            target: None,
            created_kinds: Vec::new(),
            pruned_kinds: Vec::new(),
            configured_kinds: Vec::new(),
            details: "test".to_string(),
            link: "http://example.com".to_string(),
            is_no_changes: false,
        };
        assert!(!template.is_same_build("test").unwrap());
    }

    #[test]
    fn test_is_same_build_with_same_build() {
        let template = Template {
            target: Some("test".to_string()),
            created_kinds: Vec::new(),
            pruned_kinds: Vec::new(),
            configured_kinds: Vec::new(),
            details: "test".to_string(),
            link: "http://example.com".to_string(),
            is_no_changes: false,
        };
        assert!(template.is_same_build("## Plan result (test)").unwrap())
    }

    #[test]
    fn test_is_same_build_with_different_build() {
        let template = Template {
            target: Some("test1".to_string()),
            created_kinds: Vec::new(),
            pruned_kinds: Vec::new(),
            configured_kinds: Vec::new(),
            details: "test".to_string(),
            link: "http://example.com".to_string(),
            is_no_changes: false,
        };
        assert!(!template.is_same_build("## Plan result (test2)").unwrap())
    }
}
