use indexmap::IndexMap;
use regex::Regex;

pub struct Context {
    regexp: Regex,
    variables: IndexMap<String, String>,
}

impl Context {
    pub fn new(variables: &IndexMap<String, String>) -> Self {
        let mut v: IndexMap<String, String> = IndexMap::new();
        for (key, value) in variables.iter() {
            v.insert(
                key.clone().as_str().to_lowercase(),
                value.clone().as_str().to_lowercase()
            );
        }
        // Using this results in an error message. The reason is regex pattern
        // that we are trying to use contains look-around assertions, specifically
        // a negative look-behind assertion (?<!\\). In Rust's regex crate, certain
        // look-around assertions are not supported. We added to try and skip \${VARIABLE}
        // let us see if we can manage without or we will have figure something out
        //let regexp = Regex::new(r"(?<!\\)\$\{(\w+|\{([^}]+)\})\}").unwrap();
        let regexp = Regex::new(r"\$\{(\w+|\{([^}]+)\})\}").unwrap();
        Context {
            regexp,
            variables: v,
        }
    }

    pub fn expand_str(&self, s: &str) -> String {
        let replaced = self.regexp.replace_all(
            s, |caps: &regex::Captures| {
            let var_name = &caps[1].to_lowercase(); // Extract the variable name
            match self.variables.get(var_name) {
                Some(value) => value.to_string(), // Replace with the value from the IndexMap
                None => caps[0].to_string(), // No replacement found, keep the original text
            }
        });
        replaced.to_string()
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use crate::configs::Context;
    use crate::error::BError;

    #[test]
    fn test_task_context_expand_str() {
        let mut variables: IndexMap<String, String> = IndexMap::new();
        variables.insert(String::from("VAR1"), String::from("var1"));
        variables.insert(String::from("VAR2"), String::from("var2"));
        variables.insert(String::from("VAR3"), String::from("var3"));
        let ctx: Context = Context::new(&variables);
        assert_eq!(ctx.expand_str("Testing ${VAR1} expansion"), "Testing var1 expansion");
        assert_eq!(ctx.expand_str("Testing ${VAR2} expansion"), "Testing var2 expansion");
        assert_eq!(ctx.expand_str("Testing ${VAR3} expansion"), "Testing var3 expansion");
        assert_eq!(ctx.expand_str("Testing ${VAR1} ${VAR2} ${VAR3} expansion"), "Testing var1 var2 var3 expansion");  
    }
}

