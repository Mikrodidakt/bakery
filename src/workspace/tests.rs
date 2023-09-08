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
        assert_eq!(settings.workspace_configs_dir(),  "configs_test");
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
        assert_eq!(settings.workspace_configs_dir(),  "configs");
    }

    #[test]
    fn test_settings_no_configs_workspace_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_configs_dir(),  "configs");
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
        assert_eq!(settings.workspace_builds_dir(),  "builds_test");
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
        assert_eq!(settings.workspace_builds_dir(),  "builds");
    }

    #[test]
    fn test_settings_no_builds_workspace_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_builds_dir(),  "builds");
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
        assert_eq!(settings.workspace_artifacts_dir(), "artifacts_test");
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
        assert_eq!(settings.workspace_artifacts_dir(), "artifacts");
    }

    #[test]
    fn test_settings_no_artifacts_workspace_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_artifacts_dir(), "artifacts");
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
        assert_eq!(settings.workspace_scripts_dir(), "scripts_test");
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
        assert_eq!(settings.workspace_scripts_dir(), "scripts");
    }

    #[test]
    fn test_settings_no_scripts_workspace_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_scripts_dir(), "scripts");
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
        assert_eq!(settings.workspace_docker_dir(), "docker_test");
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
        assert_eq!(settings.workspace_docker_dir(), "docker");
    }

    #[test]
    fn test_settings_no_docker_workspace_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_docker_dir(), "docker");
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
        assert_eq!(settings.workspace_cache_dir(), "cache_test");
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
        assert_eq!(settings.workspace_cache_dir(), "cache");
    }

    #[test]
    fn test_settings_no_cache_workspace_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_cache_dir(), "cache");
    }

    #[test]
    fn test_settings_docker_image() {
        let json_test_str = r#"
        {
            "version": 4,
            "docker": {
                "image": "test-workspace"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_image(), "test-workspace");
    }

    #[test]
    fn test_settings_no_docker_image() {
        let json_test_str = r#"
        {
            "version": 4,
            "docker": {
                "tag": "0.1"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_image(), "bakery-workspace");
    }

    #[test]
    fn test_settings_no_docker_image_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_image(), "bakery-workspace");
    }

    #[test]
    fn test_settings_docker_tag() {
        let json_test_str = r#"
        {
            "version": 4,
            "docker": {
                "tag": "0.1"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_tag(), "0.1");
    }

    #[test]
    fn test_settings_no_docker_tag() {
        let json_test_str = r#"
        {
            "version": 4,
            "docker": {
                "image": "test-workspace"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_tag(),  "0.68");
    }

    #[test]
    fn test_settings_no_docker_tag_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_tag(), "0.68");
    }

    #[test]
    fn test_settings_docker_registry() {
        let json_test_str = r#"
        {
            "version": 4,
            "docker": {
                "registry": "test-registry"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_registry(), "test-registry");
    }

    #[test]
    fn test_settings_no_docker_registry() {
        let json_test_str = r#"
        {
            "version": 4,
            "docker": {
                "image": "test-workspace"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_registry(), "strixos");
    }

    #[test]
    fn test_settings_no_docker_registry_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_registry(), "strixos");
    }

    #[test]
    fn test_settings_docker_args() {
        let json_test_str = r#"
        {
            "version": 4,
            "docker": {
                "args": [
                  "--rm=true",
                  "-t",
                  "--dns=8.8.8.8"
                ]
              }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_args(), [String::from("--rm=true"), String::from("-t"), String::from("--dns=8.8.8.8")]);
    }

    #[test]
    fn test_settings_no_docker_args() {
        let json_test_str = r#"
        {
            "version": 4,
            "docker": {
                "image": "test-workspace"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_args(), [String::from("--rm=true"), String::from("-t")]);
    }

    #[test]
    fn test_settings_no_docker_args_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);  
        assert_eq!(settings.docker_args(), [String::from("--rm=true"), String::from("-t")]);
    }

    #[test]
    fn test_settings_build_configs() {
        let json_test_str = r#"
        {
            "version": 4,
            "build": {
                "supported": [
                  "machine1-test",
                  "machine2-test"
                ]
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.supported_build_configs(),  vec![String::from("machine1-test"), String::from("machine2-test")]);
        let mut i: i32 = 1;
        for supported in settings.supported_build_configs() {
            assert_eq!(supported, format!("machine{}-test", i));
            i += 1;
        }
    }

    #[test]
    fn test_settings_no_supported_build_configs() {
        let json_test_str = r#"
        {
            "version": 4,
            "build": {
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.supported_build_configs().is_empty(), true);
    }

    #[test]
    fn test_settings_no_build_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.supported_build_configs().is_empty(), true);
    }
}