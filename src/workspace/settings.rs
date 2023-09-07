use serde_json::Value;
pub struct Settings {
    data: Value, 
}

impl Settings {
    pub fn from_str(json_string: &str) -> Result<Self, serde_json::Error> {
        let data: Value = serde_json::from_str(json_string)?;
        Ok(Settings { data })
    }

    pub fn print(&self) {
        println!("data {}", self.data);
    }

    pub fn workspace_configs_dir(&self) -> &str {
        &self.data["workspace"]["configsdir"].as_str().unwrap()
    }
}

