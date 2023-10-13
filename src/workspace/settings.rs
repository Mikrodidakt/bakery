use crate::{configs::WsSettings, executers::DockerImage};
use crate::error::BError;

use std::path::{PathBuf, Path};

#[derive(Clone)]
pub struct WsSettingsHandler {
    work_dir: PathBuf,
    wsSettings: WsSettings,
    docker: DockerImage,
    curr_cfg_name: String,
}

impl WsSettingsHandler {
    pub fn from_str(work_dir: &PathBuf, json_settings: &str) -> Result<Self, BError> {
        let work_dir: PathBuf = work_dir.clone();
        let result: Result<WsSettings, BError> = WsSettings::from_str(json_settings);
        match result {
            Ok(rsettings) => {
               Ok(Self::new(work_dir, rsettings))
            }
            Err(e) => {
                Err(e)
            } 
        }
    }

    pub fn new(work_dir: PathBuf, settings: WsSettings) -> Self {
        let docker = DockerImage {
            image: settings.docker_image.clone(),
            tag: settings.docker_tag.clone(),
            registry: settings.docker_registry.clone(),
        };
        WsSettingsHandler {
            work_dir,
            wsSettings: settings,
            docker,
            curr_cfg_name: "".to_string(),
        }
    }

    pub fn verify_ws_dir(&self, dir: &Path) -> Result<(), BError> {
        if !dir.is_dir() || !dir.exists() {
            return Err(BError::WsError(format!("Workspace directory '{}' dosen't exists", dir.display())));
        }
        return Ok(());
    }

    pub fn verify_ws(&self) -> Result<(), BError> {
        self.verify_ws_dir(self.configs_dir().as_path())?;
        // Some directories should be created during run time
        //self.verify_ws_dir(self.builds_dir().as_path())?;
        //self.verify_ws_dir(self.artifacts_dir().as_path())?;
        // Some directories are optional
        //self.verify_ws_dir(self.include_dir().as_path())?;
        //self.verify_ws_dir(self.scripts_dir().as_path())?;
        //self.verify_ws_dir(self.docker_dir().as_path())?;
        Ok(())
    }

    pub fn work_dir(&self) -> PathBuf {
        self.work_dir.clone()
    }

    pub fn config(&self) -> &WsSettings {
        &self.wsSettings
    }

    pub fn builds_dir(&self) -> PathBuf {
        let mut path_buf = self.work_dir();
        path_buf.push(&self.wsSettings.builds_dir);
        path_buf
    }

    pub fn cache_dir(&self) -> PathBuf {
        let mut path_buf = self.work_dir();
        path_buf.push(&self.wsSettings.cache_dir);
        path_buf
    }

    pub fn artifacts_dir(&self) -> PathBuf {
        let mut path_buf = self.work_dir();
        path_buf.push(&self.wsSettings.artifacts_dir);
        path_buf
    }

    pub fn configs_dir(&self) -> PathBuf {
        let mut path_buf = self.work_dir();
        path_buf.push(&self.wsSettings.configs_dir);
        path_buf
    }

    pub fn include_dir(&self) -> PathBuf {
        let mut path_buf = self.work_dir();
        path_buf.push(&self.wsSettings.include_dir);
        path_buf
    }

    pub fn scripts_dir(&self) -> PathBuf {
        let mut path_buf = self.work_dir();
        path_buf.push(&self.wsSettings.scripts_dir);
        path_buf
    }

    pub fn docker_dir(&self) -> PathBuf {
        let mut path_buf = self.work_dir();
        path_buf.push(&self.wsSettings.docker_dir);
        path_buf
    }

    pub fn docker_image(&self) -> DockerImage {
        self.docker.clone()
    }

    pub fn docker_args(&self) -> &Vec<String> {
        &self.wsSettings.docker_args
    }

    pub fn supported_builds(&self) -> &Vec<String> {
        &self.wsSettings.supported
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::executers::DockerImage;
    use crate::workspace::WsSettingsHandler;
    use crate::helper::Helper;

    #[test]
    fn test_settings_default_ws_dirs() {
        let json_test_str = r#"
        {
            "version": "4"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let settings: WsSettingsHandler = WsSettingsHandler::new(
            work_dir,
            Helper::setup_ws_settings(json_test_str),
        );
        assert_eq!(settings.builds_dir(), PathBuf::from("/workspace/builds"));
        assert_eq!(settings.cache_dir(), PathBuf::from("/workspace/.cache"));
        assert_eq!(settings.artifacts_dir(), PathBuf::from("/workspace/artifacts"));
        assert_eq!(settings.scripts_dir(), PathBuf::from("/workspace/scripts"));
        assert_eq!(settings.docker_dir(), PathBuf::from("/workspace/docker"));
        assert_eq!(settings.configs_dir(), PathBuf::from("/workspace/configs"));
        assert_eq!(settings.include_dir(), PathBuf::from("/workspace/configs/include"));
    }

    #[test]
    fn test_settings_ws_dirs() {
        let json_test_str = r#"
        {
            "version": "4",
            "workspace": {
              "configsdir": "configs_test",
              "includedir": "include_test",
              "artifactsdir": "artifacts_test",
              "buildsdir": "builds_test",
              "artifactsdir": "artifacts_test",
              "scriptsdir": "scripts_test",
              "dockerdir": "docker_test",
              "cachedir": "cache_test"
            }
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let settings: WsSettingsHandler = WsSettingsHandler::new(
            work_dir,
            Helper::setup_ws_settings(json_test_str),
        );
        assert_eq!(settings.builds_dir(), PathBuf::from("/workspace/builds_test"));
        assert_eq!(settings.cache_dir(), PathBuf::from("/workspace/cache_test"));
        assert_eq!(settings.artifacts_dir(), PathBuf::from("/workspace/artifacts_test"));
        assert_eq!(settings.scripts_dir(), PathBuf::from("/workspace/scripts_test"));
        assert_eq!(settings.docker_dir(), PathBuf::from("/workspace/docker_test"));
        assert_eq!(settings.configs_dir(), PathBuf::from("/workspace/configs_test"));
        assert_eq!(settings.include_dir(), PathBuf::from("/workspace/include_test"));
    }

    #[test]
    fn test_settings_default_docker() {
        let json_test_str = r#"
        {
            "version": "4"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let settings: WsSettingsHandler = WsSettingsHandler::new(
            work_dir,
            Helper::setup_ws_settings(json_test_str),
        );
        let docker_image: DockerImage = settings.docker_image();
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
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let settings: WsSettingsHandler = WsSettingsHandler::new(
            work_dir,
            Helper::setup_ws_settings(json_test_str),
        );
        let docker_image: DockerImage = settings.docker_image();
        assert_eq!(format!("{}", docker_image), "test-registry/test-image:0.1");
    }

    #[test]
    fn test_settings_default_docker_args() {
        let json_test_str = r#"
        {
            "version": "4",
            "docker": {
                "tag": "0.1",
                "image": "test-image",
                "registry": "test-registry"
            }
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let settings: WsSettingsHandler = WsSettingsHandler::new(
            work_dir,
            Helper::setup_ws_settings(json_test_str),
        );
        assert_eq!(settings.docker_args(), &vec!["--rm=true".to_string(), "-t".to_string()]);
    }

    #[test]
    fn test_settings_docker_args() {
        let json_test_str = r#"
        {
            "version": "4",
            "docker": {
                "tag": "0.1",
                "image": "test-image",
                "registry": "test-registry",
                "args": [
                    "arg1",
                    "arg2",
                    "arg3"
                ]
            }
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let settings: WsSettingsHandler = WsSettingsHandler::new(
            work_dir,
            Helper::setup_ws_settings(json_test_str),
        );
        assert_eq!(settings.docker_args(), &vec!["arg1".to_string(), "arg2".to_string(), "arg3".to_string()]);
    }

    #[test]
    fn test_settings_default_supported_builds() {
        let json_test_str = r#"
        {
            "version": "4"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let settings: WsSettingsHandler = WsSettingsHandler::new(
            work_dir,
            Helper::setup_ws_settings(json_test_str),
        );
        assert!(settings.supported_builds().is_empty());
    }

    #[test]
    fn test_settings_supported_builds() {
        let json_test_str = r#"
        {
            "version": "4",
            "builds": {
                "supported": [
                    "build1",
                    "build2"
                ]
            }
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let settings: WsSettingsHandler = WsSettingsHandler::new(
            work_dir,
            Helper::setup_ws_settings(json_test_str),
        );
        assert_eq!(settings.supported_builds(), &vec!["build1".to_string(), "build2".to_string()]);
    }
}