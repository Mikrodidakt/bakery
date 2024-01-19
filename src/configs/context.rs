use indexmap::IndexMap;
use regex::Regex;
use std::path::PathBuf;

pub struct Context {
    regexp: Regex,
    variables: IndexMap<String, String>,
}

impl Context {
    pub fn new(variables: &IndexMap<String, String>) -> Self {
        let v: IndexMap<String, String> = variables
            .into_iter()
            .map(|(key, value)|(key.to_lowercase(), value.clone()))
            .collect();
        // Using this results in an error message. The reason is regex pattern
        // that we are trying to use contains look-around assertions, specifically
        // a negative look-behind assertion (?<!\\). In Rust's regex crate, certain
        // look-around assertions are not supported. We added this negative look-behind
        // assertion to try and skip \${VARIABLE} let us see if we can manage without
        // or we will have figure something out.
        // let regexp = Regex::new(r"(?<!\\)\$\{(\w+|\{([^}]+)\})\}").unwrap();
        let regexp = Regex::new(r"\$\{(\w+|\{([^}]+)\})\}").unwrap();
        Context {
            regexp,
            variables: v,
        }
    }

    fn __expand_str(&self, s: &str) -> String {
        let replaced = self.regexp.replace_all(
            s, |caps: &regex::Captures| {
                let var_name = &caps[1].to_lowercase(); // Extract the variable name
                match self.variables.get(var_name) {
                    Some(value) => value.to_string(), // Replace with the value from the IndexMap
                    None => caps[0].to_string(), // No replacement found, keep the original text
                }
            }
        );
        replaced.to_string()
    }

    pub fn expand_str(&self, s: &str) -> String {
        let mut counter = 0;
        let mut expanded_string: String = s.to_string();
        while self.regexp.is_match(expanded_string.as_str()) {
            expanded_string = self.__expand_str(expanded_string.as_str());
            if counter > 10 {
                // TODO not sure we should panic. If we don't find a context should we panic then?
                panic!("Failed to expand context in string '{}'", expanded_string);
            }
            counter += 1;
        }
        expanded_string
    }

    pub fn expand_path(&self, p: &PathBuf) -> PathBuf {
        let p_str: String = self.expand_str(p.to_str().unwrap());
        PathBuf::from(p_str)
    }

    pub fn value(&self, key: &str) -> String {
        match self.variables.get(&key.to_lowercase()) {
            Some(value) => {
                value.clone()
            },
            None => {
                // TODO: For now we are just returning an empty String if
                // the key is invalid we should maybe consider returning
                // Result
                String::from("")
            }
        }
    }

    pub fn merge(&mut self, context: &Context) {
        self.update(&context.variables);
    }

    pub fn update(&mut self, variables: &IndexMap<String, String>) {
        self.variables.extend(variables
            .into_iter()
            .map(|(key, value)|(key.to_lowercase(), value.clone())));
    }

    pub fn variables(&self) -> &IndexMap<String, String> {
        &self.variables
    }
}

#[cfg(test)]
mod tests {
    use indexmap::{IndexMap, indexmap};
    use std::path::PathBuf;

    use crate::configs::Context;

    #[test]
    fn test_task_context_variables() {
        let variables: IndexMap<String, String> = indexmap! {
            "VAR1".to_string() => "var1".to_string(),
            "VAR2".to_string() => "var2".to_string(),
            "VAR3".to_string() => "var3".to_string(),
            "VAR4".to_string() => "${VAR1}".to_string()
        };
        let ctx: Context = Context::new(&variables);
        assert_eq!(ctx.value("VAR1"), "var1");
        assert_eq!(ctx.value("VAR2"), "var2");
        assert_eq!(ctx.value("VAR3"), "var3");
        assert_eq!(ctx.value("VAR4"), "${VAR1}");
        assert!(ctx.value("VAR5").is_empty());
    }

    #[test]
    fn test_task_context_update() {
        let variables1: IndexMap<String, String> = indexmap! {
            "DIR1".to_string() => "dir1".to_string(),
            "DIR2".to_string() => "dir2".to_string(),
            "DIR3".to_string() => "dir3".to_string()
        };
        let mut ctx: Context = Context::new(&variables1);
        let variables2: IndexMap<String, String> = indexmap! {
            "NEWDIR1".to_string() => "newdir1".to_string(),
            "NEWDIR2".to_string() => "newdir2".to_string()
        };
        ctx.update(&variables2);
        assert_eq!(ctx.value("DIR1"), "dir1");
        assert_eq!(ctx.value("DIR2"), "dir2");
        assert_eq!(ctx.value("DIR3"), "dir3");
        assert_eq!(ctx.value("NEWDIR1"), "newdir1");
        assert_eq!(ctx.value("NEWDIR2"), "newdir2");
    }

    #[test]
    fn test_task_context_expand_str() {
        let variables: IndexMap<String, String> = indexmap! {
            "VAR1".to_string() => "var1".to_string(),
            "VAR2".to_string() => "var2".to_string(),
            "VAR3".to_string() => "var3".to_string()
        };
        let ctx: Context = Context::new(&variables);
        assert_eq!(ctx.expand_str("Testing ${VAR1} expansion"), "Testing var1 expansion");
        assert_eq!(ctx.expand_str("Testing ${VAR2} expansion"), "Testing var2 expansion");
        assert_eq!(ctx.expand_str("Testing ${VAR3} expansion"), "Testing var3 expansion");
        assert_eq!(ctx.expand_str("Testing ${VAR1} ${VAR2} ${VAR3} expansion"), "Testing var1 var2 var3 expansion");
    }

    #[test]
    fn test_task_context_nested_expand_str() {
        let variables: IndexMap<String, String> = indexmap! {
            "VAR1".to_string() => "${VAR4}".to_string(),
            "VAR2".to_string() => "var2".to_string(),
            "VAR3".to_string() => "var3".to_string(),
            "VAR4".to_string() => "var4".to_string()
        };
        let ctx: Context = Context::new(&variables);
        assert_eq!(ctx.expand_str("Testing ${VAR1} expansion"), "Testing var4 expansion");
        assert_eq!(ctx.expand_str("Testing ${VAR2} expansion"), "Testing var2 expansion");
        assert_eq!(ctx.expand_str("Testing ${VAR3} expansion"), "Testing var3 expansion");
        assert_eq!(ctx.expand_str("Testing ${VAR1} ${VAR2} ${VAR3} expansion"), "Testing var4 var2 var3 expansion");
    }

    #[test]
    fn test_task_context_update_nested() {
        let variables1: IndexMap<String, String> = indexmap! {
            "DIR1".to_string() => "dir1".to_string(),
            "DIR2".to_string() => "dir2".to_string(),
            "DIR3".to_string() => "dir3".to_string()
        };
        let mut ctx: Context = Context::new(&variables1);
        let variables2: IndexMap<String, String> = indexmap! {
            "NEWDIR1".to_string() => "${DIR1}/newdir1".to_string(),
            "NEWDIR2".to_string() => "${DIR2}/newdir2".to_string()
        };
        ctx.update(&variables2);
        assert_eq!(ctx.expand_str("/dir/${NEWDIR1}/file1.txt"), "/dir/dir1/newdir1/file1.txt");
        assert_eq!(ctx.expand_str("/dir/${NEWDIR2}/file2.txt"), "/dir/dir2/newdir2/file2.txt");
    }

    #[test]
    fn test_task_context_expand_path() {
        let variables: IndexMap<String, String> = indexmap! {
            "VAR1".to_string() => "var1".to_string(),
            "VAR2".to_string() => "var2".to_string(),
            "VAR3".to_string() => "var3".to_string()
        };
        let ctx: Context = Context::new(&variables);
        let path: PathBuf = PathBuf::from("/dir1/${VAR1}/${VAR2}/${VAR3}/file1.txt");
        assert_eq!(ctx.expand_path(&path), PathBuf::from("/dir1/var1/var2/var3/file1.txt"));
    }
}

