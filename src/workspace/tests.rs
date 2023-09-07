#[cfg(test)]
mod tests {
    use crate::workspace::Settings;

    fn helper_settings_from_str(json_test_str: &str) -> Settings {
        let result: Result<Settings, serde_json::Error> = Settings::from_str(json_test_str);
        let settings: Settings;
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
    fn test_settings_configs_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "configsdir": "configs_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_configs_dir(),  "work_dir/configs_test");
    }

    #[test]
    fn test_settings_no_configs_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "artifactsdir": "artifacts_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_configs_dir(),  "work_dir/configs");
    }

    #[test]
    fn test_settings_no_configs_workspace_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_configs_dir(),  "work_dir/configs");
    }

    #[test]
    fn test_settings_builds_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "buildsdir": "builds_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_builds_dir(),  "work_dir/builds_test");
    }

    #[test]
    fn test_settings_no_builds_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "artifactsdir": "artifacts_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_builds_dir(),  "work_dir/builds");
    }

    #[test]
    fn test_settings_no_builds_workspace_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_builds_dir(),  "work_dir/builds");
    }

    #[test]
    fn test_settings_artifacts_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "artifactsdir": "artifacts_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_artifacts_dir(),  "work_dir/artifacts_test");
    }

    #[test]
    fn test_settings_no_artifacts_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "buildsdir": "builds_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_artifacts_dir(),  "work_dir/artifacts");
    }

    #[test]
    fn test_settings_no_artifacts_workspace_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_artifacts_dir(),  "work_dir/artifacts");
    }

    #[test]
    fn test_settings_scripts_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "scriptsdir": "scripts_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_scripts_dir(),  "work_dir/scripts_test");
    }

    #[test]
    fn test_settings_no_scripts_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "buildsdir": "builds_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_scripts_dir(),  "work_dir/scripts");
    }

    #[test]
    fn test_settings_no_scripts_workspace_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_scripts_dir(),  "work_dir/scripts");
    }

    #[test]
    fn test_settings_docker_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "dockerdir": "docker_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_docker_dir(),  "work_dir/docker_test");
    }

    #[test]
    fn test_settings_no_docker_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "buildsdir": "builds_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_docker_dir(),  "work_dir/docker");
    }

    #[test]
    fn test_settings_no_docker_workspace_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_docker_dir(),  "work_dir/docker");
    }

    #[test]
    fn test_settings_cache_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "cachedir": "cache_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_cache_dir(),  "work_dir/cache_test");
    }

    #[test]
    fn test_settings_no_cache_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "buildsdir": "builds_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_cache_dir(),  "work_dir/cache");
    }

    #[test]
    fn test_settings_no_cache_workspace_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_cache_dir(),  "work_dir/cache");
    }
}