use crate::{configs::SettingsConfig, executers::DockerImage};
use std::path::{PathBuf, Path};

pub struct Settings<'a> {
    workspace_dir: &'a Path,
    config: &'a SettingsConfig,
    docker: DockerImage,
}

impl<'a> Settings<'a> {
    pub fn new(workspace_dir: &'a str, config: &'a SettingsConfig) -> Self {
        Settings {
            workspace_dir: &Path::new(workspace_dir),
            config,
            docker: DockerImage {
                image: config.docker_image.clone(),
                tag: config.docker_tag.clone(),
                registry: config.docker_registry.clone(),
            }
        }
    }

    pub fn config(&self) -> &SettingsConfig {
        &self.config
    }

    pub fn builds_dir(&self) -> PathBuf {
        let mut path_buf = self.workspace_dir.to_path_buf();
        path_buf.push(&self.config.builds_dir);
        path_buf
    }

    pub fn cache_dir(&self) -> PathBuf {
        let mut path_buf = self.workspace_dir.to_path_buf();
        path_buf.push(&self.config.cache_dir);
        path_buf
    }

    pub fn artifacts_dir(&self) -> PathBuf {
        let mut path_buf = self.workspace_dir.to_path_buf();
        path_buf.push(&self.config.artifacts_dir);
        path_buf
    }

    pub fn configs_dir(&self) -> PathBuf {
        let mut path_buf = self.workspace_dir.to_path_buf();
        path_buf.push(&self.config.configs_dir);
        path_buf
    }

    pub fn scripts_dir(&self) -> PathBuf {
        let mut path_buf = self.workspace_dir.to_path_buf();
        path_buf.push(&self.config.scripts_dir);
        path_buf
    }

    pub fn docker_dir(&self) -> PathBuf {
        let mut path_buf = self.workspace_dir.to_path_buf();
        path_buf.push(&self.config.docker_dir);
        path_buf
    }

    pub fn docker_image(&self) -> &DockerImage {
        &self.docker
    }
}

#[cfg(test)]
mod tests {
    use crate::executers::DockerImage;
    use crate::workspace::Settings;
    use crate::configs::SettingsConfig;
    use crate::error::BError;

    fn helper_settings_from_str(json_test_str: &str) -> SettingsConfig {
        let result: Result<SettingsConfig, BError> = SettingsConfig::from_str(json_test_str);
        let settings: SettingsConfig;
        match result {
            Ok(rsettings) => {
                settings = rsettings;
            }
            Err(e) => {
                eprintln!("Error parsing JSON: {}", e);
                panic!();
            } 
        }
        settings
    }

    #[test]
    fn test_settings_default_ws_dirs() {
        let json_test_str = r#"
        {
            "version": "4"
        }"#;
        let config: SettingsConfig = helper_settings_from_str(json_test_str);
        let settings: Settings = Settings::new("/workspace", &config);
        assert_eq!(settings.builds_dir().to_str(), Some("/workspace/builds"));
        assert_eq!(settings.cache_dir().to_str(), Some("/workspace/.cache"));
        assert_eq!(settings.artifacts_dir().to_str(), Some("/workspace/artifacts"));
        assert_eq!(settings.scripts_dir().to_str(), Some("/workspace/scripts"));
        assert_eq!(settings.docker_dir().to_str(), Some("/workspace/docker"));
        assert_eq!(settings.configs_dir().to_str(), Some("/workspace/configs"));
    }

    #[test]
    fn test_settings_ws_dirs() {
        let json_test_str = r#"
        {
            "version": "4",
            "workspace": {
              "configsdir": "configs_test",
              "artifactsdir": "artifacts_test",
              "buildsdir": "builds_test",
              "artifactsdir": "artifacts_test",
              "scriptsdir": "scripts_test",
              "dockerdir": "docker_test",
              "cachedir": "cache_test"
            }
        }"#;
        let config: SettingsConfig = helper_settings_from_str(json_test_str);
        let settings: Settings = Settings::new("/workspace", &config);
        assert_eq!(settings.builds_dir().to_str(), Some("/workspace/builds_test"));
        assert_eq!(settings.cache_dir().to_str(), Some("/workspace/cache_test"));
        assert_eq!(settings.artifacts_dir().to_str(), Some("/workspace/artifacts_test"));
        assert_eq!(settings.scripts_dir().to_str(), Some("/workspace/scripts_test"));
        assert_eq!(settings.docker_dir().to_str(), Some("/workspace/docker_test"));
        assert_eq!(settings.configs_dir().to_str(), Some("/workspace/configs_test"));
    }

    #[test]
    fn test_settings_default_docker() {
        let json_test_str = r#"
        {
            "version": "4"
        }"#;
        let config: SettingsConfig = helper_settings_from_str(json_test_str);
        let settings: Settings = Settings::new("/workspace", &config);
        let docker_image: &DockerImage = settings.docker_image();
        assert_eq!(format!("{}", docker_image), "strixos/bakery-workspace:0.68");
    }

    #[test]
    fn test_settings_docker() {
        let json_test_str = r#"
        {
            "version": "4",
            "docker": {
                "tag": "0.1",
                "image": "test-image",
                "registry": "test-registry"
            }
        }"#;
        let config: SettingsConfig = helper_settings_from_str(json_test_str);
        let settings: Settings = Settings::new("/workspace", &config);
        let docker_image: &DockerImage = settings.docker_image();
        assert_eq!(format!("{}", docker_image), "test-registry/test-image:0.1");
    }
}