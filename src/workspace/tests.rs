#[cfg(test)]
mod tests {
    use crate::workspace::Settings;

    #[test]
    fn test_settings_build_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "configsdir": "configs_test",
              "buildsdir": "builds_test",
              "artifactsdir": "artifacts_test",
              "scriptsdir": "scripts_test",
              "dockerdir": "scripts/docker_test",
              "cachedir": ".cache_test"
            },
            "build": {
              "supported": [
                "machine1-test",
                "machine2-test"
              ]
            },
            "docker": {
              "registry": "test",
              "image": "test-workspace",
              "tag": "0.1",
              "args": [
                "--rm=true",
                "-t",
                "--dns=8.8.8.8"
              ]
            }
          }
        "#;
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
        assert_eq!(settings.workspace_configs_dir(), "configs_test");
    }
}