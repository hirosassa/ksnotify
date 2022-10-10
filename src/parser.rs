use anyhow::Result;
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
}

impl DiffParser {
    pub fn new() -> Result<Self> {
        let kind = Regex::new(r"(?m)^diff -uN\s.*/(?P<kind>[^/\s]+)\s.*/([^/\s]+)$")?; // matches line like "diff -uN /var/folders/fl/blahblah/[apiVersion].[kind].[namespace].[name]  /var/folders/fl/blahblah/[apiVersion].[kind].[namespace].[name]"
        let header = Regex::new(r"(?m)^.*/var/folders/.*$")?; // matches diff header that contains "/var/folders/fl/blahblah/"
        Ok(Self { kind, header })
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
}

impl Parsable for DiffParser {
    fn parse(&self, diff: &str) -> Result<ParseResult> {
        let kinds = self.parse_kinds(diff);
        let chunked_diff = self.parse_diff(diff);
        let result: HashMap<_, _> = kinds
            .iter()
            .zip(chunked_diff.iter())
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

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
        let diff = "diff -uN /var/folders/fl/blahblah/v1.Service.test.test-app1 /var/folders/fl/blahblah/v1.Service.test.test-app1
--- /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
+++ /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
ABCDE
FGHIJ
diff -uN /var/folders/fl/blahblah/v1.Service.test.test-app2 /var/folders/fl/blahblah/v1.Service.test.test-app2
--- /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
+++ /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
12345
67890";
        let parser = self::DiffParser::new().unwrap();
        let actual = parser.parse(&diff).unwrap();
        //assert_eq!(actual.kind_result.len(), 2);

        let keys = vec!["v1.Service.test.test-app1", "v1.Service.test.test-app2"];
        let values = vec!["ABCDE\nFGHIJ", "12345\n67890"];
        for (k, v) in keys.iter().zip(values) {
            assert_eq!(actual.kind_result[&k.to_string()], v.to_string())
        }
    }

    #[test]
    fn test_parse_kinds_correctly_extracts_kind() {
        let diff = "diff -uN /var/folders/fl/blahblah/v1.Service.test.test-app1 /var/folders/fl/blahblah/v1.Service.test.test-app1
--- /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
+++ /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
ABCDE
FGHIJ
diff -uN /var/folders/fl/blahblah/v1.Service.test.test-app2 /var/folders/fl/blahblah/v1.Service.test.test-app2
--- /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
+++ /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
12345
67890";

        let parser = self::DiffParser::new().unwrap();
        let actual = parser.parse_kinds(&diff);
        let expected = vec!["v1.Service.test.test-app1", "v1.Service.test.test-app2"];
        assert_eq!(&actual[..], &expected[..]);
    }

    #[test]
    fn test_parse_diff_correctly_extracts_list_of_diff_body() {
        let diff = "diff -uN /var/folders/fl/blahblah/v1.Service.test.test-app /var/folders/fl/blahblah/v1.Service.test.test-app
--- /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
+++ /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
ABCDE
FGHIJ
diff -uN /var/folders/fl/blahblah/v1.Service.test.test-app2 /var/folders/fl/blahblah/v1.Service.test.test-app2
--- /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
+++ /var/folders/fl/blahblah/v1.Service.test.test-app	2022-02-22 22:00:00.000000000 +0900
12345
67890";
        let parser = self::DiffParser::new().unwrap();
        let actual = parser.parse_diff(&diff);
        let expected = vec!["ABCDE\nFGHIJ", "12345\n67890"];
        assert_eq!(&actual[..], &expected[..]);
    }
}
