pub struct Recipe {
    recipe: String,
    task: String,
}

impl Recipe {
    pub fn new(recipe_str: &str) -> Self {
        let r: Vec<&str> = recipe_str.split(':').collect();
        let mut recipe: String = String::from("");
        let mut task: String = String::from("");

        if r.len() == 1 {
            recipe = r.get(0).unwrap().to_string();
        } else if r.len() == 2 {
            recipe = r.get(0).unwrap().to_string();
            task = r.get(1).unwrap().to_string();
            if task.eq("sdk") {
                task = String::from("do_populate_sdk");
            }
            if !task.starts_with("do_") {
                task = "do_".to_string() + &task;
            }
        }

        Recipe {
            recipe: recipe.to_string(),
            task: task.to_string(),
        }
    }

    pub fn bitbake_cmd(&self) -> Vec<String> {
        let mut cmd: Vec<String> = vec!["bitbake".to_string(), self.recipe.clone()];
        if !self.task.is_empty() {
            cmd.extend(vec!["-c".to_string(), self.task.clone()]);
        }
        cmd
    }
}

#[cfg(test)]
mod tests {
    use crate::executers::Recipe;

    #[test]
    fn test_recipe_no_task() {
        let recipe: Recipe = Recipe::new("test-image");
        assert_eq!(recipe.bitbake_cmd(), vec!["bitbake", "test-image"]);
    }

    #[test]
    fn test_recipe_sdk() {
        let recipe: Recipe = Recipe::new("test-image:sdk");
        assert_eq!(
            recipe.bitbake_cmd(),
            vec!["bitbake", "test-image", "-c", "do_populate_sdk"]
        );
    }

    #[test]
    fn test_recipe_do_task() {
        let recipe: Recipe = Recipe::new("test-image:do_test_task");
        assert_eq!(
            recipe.bitbake_cmd(),
            vec!["bitbake", "test-image", "-c", "do_test_task"]
        );
    }

    #[test]
    fn test_recipe_task() {
        let recipe: Recipe = Recipe::new("test-image:test");
        assert_eq!(
            recipe.bitbake_cmd(),
            vec!["bitbake", "test-image", "-c", "do_test"]
        );
    }
}
