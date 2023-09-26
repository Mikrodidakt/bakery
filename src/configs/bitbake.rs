use serde_json::Value;
use crate::configs::{Config, Context};
use crate::error::BError;

pub struct BBConfig {
    pub machine: String, // Optional but if there is a task with type bitbake defined it might fail
    pub distro: String, // Optional but if there is a task with type bitbake defined it might fail
    pub deploy_dir: String, // Optional if not set the default deploy dir will be used builds/tmp/deploydir
    pub docker: String, // Optional if nothing is set the bitbake task will be executed inside the bakery container. Default is an empty string
    pub bblayers_conf: Vec<String>, // Optional but if there is a task with type bitbake defined it will fail without a bblayers.conf
    pub local_conf: Vec<String>, // Optional but if there is a task with type bitbake defined it will fail without a local.conf
}

impl Config for BBConfig {
}

impl BBConfig {
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
        Ok(BBConfig {
            machine,
            distro,
            docker,
            deploy_dir,
            bblayers_conf,
            local_conf,
        })
    }

    pub fn expand_ctx(&mut self, ctx: &Context) {
        self.machine = ctx.expand_str(&self.machine);
        self.distro = ctx.expand_str(&self.distro);
        self.docker = ctx.expand_str(&self.docker);
        self.deploy_dir = ctx.expand_str(&self.deploy_dir);
        // We should never expand context in the bblayers_conf and local_conf since these
        // files will handle it's own context using the bitbake variables with the same format ${}
        // we should potentially consider to have some other format for bakery context variables
        // it would make it possible to use context inside the bitbake config files.
    }
}

#[cfg(test)]
mod tests {
    use crate::configs::{BBConfig, Context};
    use crate::error::BError;

    use indexmap::{IndexMap, indexmap};

    fn helper_bitbake_config_from_str(json_test_str: &str) -> BBConfig {
        let result: Result<BBConfig, BError> = BBConfig::from_str(json_test_str);
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
        let json_test_str: &str = r#"
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
        let config: BBConfig = helper_bitbake_config_from_str(json_test_str);
        assert_eq!(config.machine, "test-machine");
        assert_eq!(config.distro, "test-distro");
        assert_eq!(config.deploy_dir, "tmp/test/deploy");
        assert_eq!(&config.bblayers_conf, &vec![
            String::from("BB_LAYERS_CONF_TEST_LINE_1"),
            String::from("BB_LAYERS_CONF_TEST_LINE_2"),
            String::from("BB_LAYERS_CONF_TEST_LINE_3")
        ]);
        assert_eq!(&config.local_conf, &vec![
            String::from("BB_LOCAL_CONF_TEST_LINE_1"),
            String::from("BB_LOCAL_CONF_TEST_LINE_2"),
            String::from("BB_LOCAL_CONF_TEST_LINE_3")
        ]);
    }

    #[test]
    fn test_bb_config_empty() {
        let json_test_str: &str = r#"
        {
        }
        "#;
        let config: BBConfig = helper_bitbake_config_from_str(json_test_str);
        assert_eq!(config.machine, "");
        assert_eq!(config.distro, "");
        assert_eq!(config.deploy_dir, "tmp/deploy/images");
        assert!(config.bblayers_conf.is_empty());
        assert!(config.local_conf.is_empty());
    }

    #[test]
    fn test_bb_config_backslash() {
        let json_test_str: &str = r#"
        {
            "machine": "test-machine",                                                                                           
            "distro": "test-distro",
            "deploydir": "tmp/test/deploy",
            "docker": "strixos/bakery-workspace:0.68",
            "bblayersconf": [
                "BBPATH=\"${TOPDIR}\"",
                "STRIXOS_WORKSPACE := \"${@os.path.abspath(os.path.dirname(d.getVar('FILE', True)) + '/../../..')}\"",
                "BBLAYERS ?= \" \\",
                "   ${STRIXWORKDIR}/layers/poky/meta \\"
            ],
            "localconf": [
                "BB_NUMBER_THREADS ?= \"${@oe.utils.cpu_count()}\"", 
                "INHERIT += \"rm_work\"",
                "BB_DISKMON_DIRS = \" \\ ",  
                "   STOPTASKS,${TMPDIR},1G,100K \\ ",
                "   ABORT,/tmp,10M,1K \"" 
            ]
        }"#;
        let config: BBConfig = helper_bitbake_config_from_str(json_test_str);
        assert_eq!(config.machine, "test-machine");
        assert_eq!(config.distro, "test-distro");
        assert_eq!(config.deploy_dir, "tmp/test/deploy");
        assert_eq!(&config.bblayers_conf, &vec![
            String::from("BBPATH=\"${TOPDIR}\""),
            String::from("STRIXOS_WORKSPACE := \"${@os.path.abspath(os.path.dirname(d.getVar('FILE', True)) + '/../../..')}\""),
            String::from("BBLAYERS ?= \" \\"),
            String::from("   ${STRIXWORKDIR}/layers/poky/meta \\"),
        ]);
        assert_eq!(&config.local_conf, &vec![
            String::from("BB_NUMBER_THREADS ?= \"${@oe.utils.cpu_count()}\""),
            String::from("INHERIT += \"rm_work\""),
            String::from("BB_DISKMON_DIRS = \" \\ "),
            String::from("   STOPTASKS,${TMPDIR},1G,100K \\ "),
            String::from("   ABORT,/tmp,10M,1K \""),
        ]);
    }

    #[test]
    fn test_bb_config_expand_context() {
        let variables: IndexMap<String, String> = indexmap! {
            "TEST_DEPLOY_DIR".to_string() => "tmp/test/dir".to_string(),
            "TEST_DOCKER_REG".to_string() => "test-registry".to_string(),
            "TEST_DOCKER_IMAGE".to_string() => "test-image".to_string(),
            "TEST_DOCKER_TAG".to_string() => "0.4".to_string(),
            "TEST_MACHINE".to_string() => "machine".to_string(),
            "TEST_DISTRO".to_string() => "distro".to_string(),
        };
        let ctx: Context = Context::new(&variables);
        let json_test_str: &str = r#"
        {
            "machine": "test-${TEST_MACHINE}",                                                                                           
            "distro": "test-${TEST_DISTRO}",
            "deploydir": "${TEST_DEPLOY_DIR}/deploy",
            "docker": "${TEST_DOCKER_REG}/${TEST_DOCKER_IMAGE}:${TEST_DOCKER_TAG}",
            "bblayersconf": [
                "BBPATH=\"${TOPDIR}\"",
                "STRIXOS_WORKSPACE := \"${@os.path.abspath(os.path.dirname(d.getVar('FILE', True)) + '/../../..')}\"",
                "BBLAYERS ?= \" \\",
                "   ${STRIXWORKDIR}/layers/poky/meta \\"
            ],
            "localconf": [
                "BB_NUMBER_THREADS ?= \"${@oe.utils.cpu_count()}\"", 
                "INHERIT += \"rm_work\"",
                "BB_DISKMON_DIRS = \" \\ ",  
                "   STOPTASKS,${TMPDIR},1G,100K \\ ",
                "   ABORT,/tmp,10M,1K \"" 
            ]
        }"#;
        let mut config: BBConfig = helper_bitbake_config_from_str(json_test_str);
        config.expand_ctx(&ctx);
        assert_eq!(config.machine, "test-machine");
        assert_eq!(config.distro, "test-distro");
        assert_eq!(config.deploy_dir, "tmp/test/dir/deploy");
        assert_eq!(config.docker, "test-registry/test-image:0.4");
        assert_eq!(&config.bblayers_conf, &vec![
            String::from("BBPATH=\"${TOPDIR}\""),
            String::from("STRIXOS_WORKSPACE := \"${@os.path.abspath(os.path.dirname(d.getVar('FILE', True)) + '/../../..')}\""),
            String::from("BBLAYERS ?= \" \\"),
            String::from("   ${STRIXWORKDIR}/layers/poky/meta \\"),
        ]);
        assert_eq!(&config.local_conf, &vec![
            String::from("BB_NUMBER_THREADS ?= \"${@oe.utils.cpu_count()}\""),
            String::from("INHERIT += \"rm_work\""),
            String::from("BB_DISKMON_DIRS = \" \\ "),
            String::from("   STOPTASKS,${TMPDIR},1G,100K \\ "),
            String::from("   ABORT,/tmp,10M,1K \""),
        ]);
    }
}