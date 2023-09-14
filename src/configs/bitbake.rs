use serde_json::Value;
use crate::configs::Config;
use crate::error::BError;

pub struct BitbakeConfig {
    machine: String, // Optional but if there is a task with type bitbake defined it might fail
    distro: String, // Optional but if there is a task with type bitbake defined it might fail
    deploy_dir: String, // Optional if not set the default deploy dir will be used builds/tmp/deploydir
    docker: String, // Optional if nothing is set the bitbake task will be executed inside the bakery container. Default is an empty string
    bblayers_conf: Vec<String>, // Optional but if there is a task with type bitbake defined it will fail without a bblayers.conf
    local_conf: Vec<String>, // Optional but if there is a task with type bitbake defined it will fail without a local.conf
}

impl Config for BitbakeConfig {
}

impl BitbakeConfig {
    pub fn from_str(json_string: &str) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        Self::from_value(&data)
    }

    pub fn from_value(data: &Value) -> Result<Self, BError> {
        let machine: String = Self::get_str_value("machine", data, Some(String::from("")))?;
        let distro: String = Self::get_str_value("distro", data, Some(String::from("")))?;
        let docker: String = Self::get_str_value("docker", data, Some(String::from("")))?;
        let deploy_dir: String = Self::get_str_value("deploydir", data, Some(String::from("tmp/deploy/images")))?;
        let bblayers_conf: Vec<String> = Self::get_array_value("bblayersconf", data, Some(vec![]))?;
        let local_conf: Vec<String> = Self::get_array_value("localconf", data, Some(vec![]))?;
        Ok(BitbakeConfig {
            machine,
            distro,
            docker,
            deploy_dir,
            bblayers_conf,
            local_conf,
        })
    }

    pub fn machine(&self) -> &String {
        &self.machine
    }

    pub fn distro(&self) -> &String {
        &self.distro
    }

    pub fn deploy_dir(&self) -> &String {
        &self.deploy_dir
    }
    
    pub fn docker(&self) -> &String {
        &self.docker
    }
    
    pub fn bblayers_conf(&self) -> &Vec<String> {
        &self.bblayers_conf
    }

    pub fn local_conf(&self) -> &Vec<String> {
        &self.local_conf
    }
}

#[cfg(test)]
mod tests {
    use crate::configs::BitbakeConfig;
    use crate::error::BError;

    fn helper_bitbake_config_from_str(json_test_str: &str) -> BitbakeConfig {
        let result: Result<BitbakeConfig, BError> = BitbakeConfig::from_str(json_test_str);
        match result {
            Ok(rconfig) => {
                rconfig
            }
            Err(e) => {
                eprintln!("Error parsing bitbake config: {}", e);
                panic!();
            } 
        }
    }

    #[test]
    fn test_bb_config() {
        let json_test_str = r#"
        {
            "machine": "test-machine",                                                                                           
            "distro": "test-distro",
            "deploydir": "tmp/test/deploy",
            "docker": "strixos/bakery-workspace:0.68",
            "bblayersconf": [
                "BB_LAYERS_CONF_TEST_LINE_1",
                "BB_LAYERS_CONF_TEST_LINE_2",
                "BB_LAYERS_CONF_TEST_LINE_3"
            ],
            "localconf": [
                "BB_LOCAL_CONF_TEST_LINE_1",
                "BB_LOCAL_CONF_TEST_LINE_2",
                "BB_LOCAL_CONF_TEST_LINE_3"
            ]
        }"#;
        let config = helper_bitbake_config_from_str(json_test_str);
        assert_eq!(config.machine(), "test-machine");
        assert_eq!(config.distro(), "test-distro");
        assert_eq!(config.deploy_dir(), "tmp/test/deploy");
        assert_eq!(config.bblayers_conf(), &vec![
            String::from("BB_LAYERS_CONF_TEST_LINE_1"),
            String::from("BB_LAYERS_CONF_TEST_LINE_2"),
            String::from("BB_LAYERS_CONF_TEST_LINE_3")
        ]);
        assert_eq!(config.local_conf(), &vec![
            String::from("BB_LOCAL_CONF_TEST_LINE_1"),
            String::from("BB_LOCAL_CONF_TEST_LINE_2"),
            String::from("BB_LOCAL_CONF_TEST_LINE_3")
        ]);
    }

    #[test]
    fn test_bb_config_empty() {
        let json_test_str = r#"
        {
        }
        "#;
        let config = helper_bitbake_config_from_str(json_test_str);
        assert_eq!(config.machine(), "");
        assert_eq!(config.distro(), "");
        assert_eq!(config.deploy_dir(), "tmp/deploy/images");
        assert!(config.bblayers_conf().is_empty());
        assert!(config.local_conf().is_empty());
    }
}