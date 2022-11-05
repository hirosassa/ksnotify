use anyhow::Result;
use log::debug;
use regex::Regex;
use std::collections::HashMap;

pub trait Parsable {
    fn parse(&self, body: &str) -> Result<ParseResult>;
}

pub struct ParseResult {
    pub kind_result: HashMap<String, String>,
}

pub struct DiffParser {
    kind: Regex,
    header: Regex,
    diff: Regex,
    skaffold: Regex,
    suppress_skaffold: bool,
    generation: Regex,
}

impl DiffParser {
    pub fn new(suppress_skaffold: bool) -> Result<Self> {
        let kind = Regex::new(r"(?m)^diff -u -N\s.*/(?P<kind>[^/\s]+)\s.*/([^/\s]+)$")?; // matches line like "diff -u -N /var/folders/fl/blahblah/[apiVersion].[kind].[namespace].[name]  /var/folders/fl/blahblah/[apiVersion].[kind].[namespace].[name]"
        let header = Regex::new(r"(?m)^((diff -u -N)|(\-\-\-)|(\+\+\+)).*$")?; // matches diff header that starts with "diff -u -N" or "---" or "+++"
        let diff = Regex::new(r"(?m)^[\-\+].*$")?;
        let skaffold = Regex::new(r"(?m)^(.*labels:.*\r?\n?)?.*skaffold.dev/run-id.*\r?\n?")?;
        let generation = Regex::new(r"(?m)^.*generation: \d+.*\r?\n?")?;
        Ok(Self {
            kind,
            header,
            diff,
            skaffold,
            suppress_skaffold,
            generation,
        })
    }

    fn parse_kinds(&self, diff: &str) -> Vec<String> {
        self.kind
            .captures_iter(diff)
            .filter_map(|cap| Some(cap.name("kind")?.as_str().to_string()))
            .collect()
    }

    fn parse_diff(&self, diff: &str) -> Vec<String> {
        let temp = self.header.replace_all(diff, "DELIMITER").to_string();
        temp.split("DELIMITER")
            .filter(|&x| !x.trim().is_empty())
            .map(|x| x.trim().to_string())
            .collect()
    }

    fn suppress_skaffold_labels(&self, result: HashMap<String, String>) -> HashMap<String, String> {
        result
            .iter()
            .map(|(kind, diff)| (kind, self.remove_skaffold_labels(diff)))
            .filter(|(_kind, diff)| self.is_there_any_diff(diff))
            .map(|(kind, diff)| (kind.to_string(), diff))
            .collect()
    }

    fn suppress_generation_fields(
        &self,
        result: HashMap<String, String>,
    ) -> HashMap<String, String> {
        result
            .iter()
            .map(|(kind, diff)| (kind, self.remove_generation_fields(diff)))
            .filter(|(_kind, diff)| self.is_there_any_diff(diff))
            .map(|(kind, diff)| (kind.to_string(), diff))
            .collect()
    }

    fn remove_skaffold_labels(&self, diff: &str) -> String {
        self.skaffold.replace_all(diff, "").to_string()
    }

    fn remove_generation_fields(&self, diff: &str) -> String {
        self.generation.replace_all(diff, "").to_string()
    }

    fn is_there_any_diff(&self, body: &str) -> bool {
        self.diff.is_match(body)
    }
}

impl Parsable for DiffParser {
    fn parse(&self, diff: &str) -> Result<ParseResult> {
        let kinds = self.parse_kinds(diff);
        debug!("kinds: {:?}", kinds);
        let chunked_diff = self.parse_diff(diff);
        debug!("chunked diff: {:?}", chunked_diff);

        let mut result: HashMap<_, _> = kinds
            .iter()
            .zip(chunked_diff.iter())
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        result = self.suppress_generation_fields(result);

        if self.suppress_skaffold {
            result = self.suppress_skaffold_labels(result);
        }
        debug!("result: {:?}", result);

        Ok(ParseResult {
            kind_result: result,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_correctly_parse_diff() {
        let diff = "diff -u -N /var/folders/fl/blahblah/v1.Service.test.test-app1 /var/folders/fl/blahblah/v1.Service.test.test-app1
--- /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
+++ /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
ABCDE
FGHIJ
diff -u -N /var/folders/fl/blahblah/v1.Service.test.test-app2 /var/folders/fl/blahblah/v1.Service.test.test-app2
--- /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
+++ /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
12345
67890";
        let parser = self::DiffParser::new(false).unwrap();
        let actual = parser.parse(diff).unwrap();
        assert_eq!(actual.kind_result.len(), 2);

        let keys = vec!["v1.Service.test.test-app1", "v1.Service.test.test-app2"];
        let values = vec!["ABCDE\nFGHIJ", "12345\n67890"];
        for (k, v) in keys.iter().zip(values) {
            assert_eq!(actual.kind_result[&k.to_string()], v.to_string())
        }
    }

    #[test]
    fn test_parse_kinds_correctly_extracts_kind() {
        let diff = "diff -u -N /var/folders/fl/blahblah/v1.Service.test.test-app1 /var/folders/fl/blahblah/v1.Service.test.test-app1
--- /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
+++ /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
ABCDE
FGHIJ
diff -u -N /var/folders/fl/blahblah/v1.Service.test.test-app2 /var/folders/fl/blahblah/v1.Service.test.test-app2
--- /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
+++ /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
12345
67890";

        let parser = self::DiffParser::new(false).unwrap();
        let actual = parser.parse_kinds(diff);
        let expected = vec!["v1.Service.test.test-app1", "v1.Service.test.test-app2"];
        assert_eq!(&actual[..], &expected[..]);
    }

    #[test]
    fn test_parse_diff_correctly_extracts_list_of_diff_body() {
        let diff = "diff -u -N /var/folders/fl/blahblah/v1.Service.test.test-app /var/folders/fl/blahblah/v1.Service.test.test-app
--- /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
+++ /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
ABCDE
FGHIJ
diff -u -N /var/folders/fl/blahblah/v1.Service.test.test-app2 /var/folders/fl/blahblah/v1.Service.test.test-app2
--- /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
+++ /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
12345
67890";
        let parser = self::DiffParser::new(false).unwrap();
        let actual = parser.parse_diff(diff);
        let expected = vec!["ABCDE\nFGHIJ", "12345\n67890"];
        assert_eq!(&actual[..], &expected[..]);
    }

    #[test]
    fn test_is_there_any_diff_detect_existence_of_diff() {
        let diff = "abc
def
- hij
+ klm";
        let parser = self::DiffParser::new(false).unwrap();
        let actual = parser.is_there_any_diff(diff);
        assert!(actual);
    }

    #[test]
    fn test_is_there_any_diff_detect_non_existence_of_diff() {
        let diff = "abc
def
hij";
        let parser = self::DiffParser::new(false).unwrap();
        let actual = parser.is_there_any_diff(diff);
        assert!(!actual);
    }

    #[test]
    fn test_remove_skaffold_labels_removes_skaffold_labels() {
        let diff = "
 @@ -5,7 +5,6 @@
     deployment.kubernetes.io/revision: 1
   labels:
     app: test-app
-    skaffold.dev/run-id: 123
   name: test-app
   namespace: test
";
        let parser = self::DiffParser::new(true).unwrap();
        let actual = parser.remove_skaffold_labels(diff);
        let expected = "
 @@ -5,7 +5,6 @@
     deployment.kubernetes.io/revision: 1
   labels:
     app: test-app
   name: test-app
   namespace: test
";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_remove_skaffold_labels_do_nothing() {
        let diff = "
 @@ -5,7 +5,6 @@
     deployment.kubernetes.io/revision: 1
   labels:
     app: test-app
   name: test-app
   namespace: test
";
        let parser = self::DiffParser::new(true).unwrap();
        let actual = parser.remove_skaffold_labels(diff);
        let expected = "
 @@ -5,7 +5,6 @@
     deployment.kubernetes.io/revision: 1
   labels:
     app: test-app
   name: test-app
   namespace: test
";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_remove_skaffold_labels_removes_labels_key_and_value() {
        let diff = "
 @@ -1,8 +1,6 @@
 apiVersion: batch/v1beta1
 kind: CronJob
 metadata:
-  labels:
-    skaffold.dev/run-id: 123
   name: test-app
   namespace: test
 spec:
";
        let parser = self::DiffParser::new(true).unwrap();
        let actual = parser.remove_skaffold_labels(diff);
        let expected = "
 @@ -1,8 +1,6 @@
 apiVersion: batch/v1beta1
 kind: CronJob
 metadata:
   name: test-app
   namespace: test
 spec:
";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_remove_skaffold_labels_removes_skaffold_labels_with_and_without_label_key() {
        let diff = "
 @@ -1,8 +1,6 @@
 metadata:
   labels:
     app: test-app
-    skaffold.dev/run-id: 1234
   name: test-app
   namespace: test
 spec:
@@ -18,8 +16,6 @@
           creationTimestamp: null
-          labels:
-            skaffold.dev/run-id: 123
         spec:
           containers:
";
        let parser = self::DiffParser::new(true).unwrap();
        let actual = parser.remove_skaffold_labels(diff);
        let expected = "
 @@ -1,8 +1,6 @@
 metadata:
   labels:
     app: test-app
   name: test-app
   namespace: test
 spec:
@@ -18,8 +16,6 @@
           creationTimestamp: null
         spec:
           containers:
";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_remove_generation_fields_removes_generation_fields() {
        let diff = "
@@ -5,9 +5,7 @@
-  generation: 18
+  generation: 19
   name: test-app
   namespace: test
";
        let parser = self::DiffParser::new(true).unwrap();
        let actual = parser.remove_generation_fields(diff);
        let expected = "
@@ -5,9 +5,7 @@
   name: test-app
   namespace: test
";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_remove_generation_fields_do_nothing() {
        let diff = "
@@ -5,9 +5,7 @@
   name: test-app
   namespace: test
";
        let parser = self::DiffParser::new(true).unwrap();
        let actual = parser.remove_generation_fields(diff);
        let expected = "
@@ -5,9 +5,7 @@
   name: test-app
   namespace: test
";
        assert_eq!(actual, expected);
    }
}
