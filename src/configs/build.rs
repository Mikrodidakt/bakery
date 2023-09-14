/*
The current format of the build config would look something like this
{                                                                                                                   
    "version": 4,
    "name": "raspberrypi3",
    "description": "Raspberrypi 3",
    "arch": "armv7",
    "bb": { 
        "machine": "raspberrypi3",                                                                                           
        "distro": "poky",
        "bblayersconf": [
            "LCONF_VERSION=\"7\"",
            "BBPATH=\"${TOPDIR}\"",
            "STRIXOS_WORKSPACE := \"${@os.path.abspath(os.path.dirname(d.getVar('FILE', True)) + '/../../..')}\"",
            "STRIXOS_LAYER := \"${STRIXOS_WORKSPACE}/layers/meta-newport\"",
            "BBFILES ?= \"\"",
            "BBLAYERS ?= \" \\",
            "   ${STRIXWORKDIR}/layers/meta-strix-raspberrypi \\",
            "   ${STRIXWORKDIR}/layers/poky/meta \\",
            "   ${STRIXWORKDIR}/layers/poky/meta-poky \\", 
            "   ${STRIXWORKDIR}/layers/poky/meta-yocto-bsp \\",
            "   ${STRIXWORKDIR}/layers/meta-openembedded/meta-oe \\",
            "   ${STRIXWORKDIR}/layers/meta-openembedded/meta-networking \\", 
            "   ${STRIXWORKDIR}/layers/meta-openembedded/meta-filesystems \\",
            "   ${STRIXWORKDIR}/layers/meta-openembedded/meta-python \\",
            "   ${STRIXWORKDIR}/layers/meta-raspberrypi \"" 
        ],
        "localconf": [
            "BB_NUMBER_THREADS ?= \"${@oe.utils.cpu_count()}\"",                                                        
            "PARALLEL_MAKE ?= \"-j ${@oe.utils.cpu_count()}\"",                                                         
            "RM_OLD_IMAGE ?= \"1\"",  
            "INHERIT += \"rm_work\"", 
            "CONF_VERSION = \"1\"",  
            "PACKAGE_CLASSES = \"package_rpm\"", 
            "SDKMACHINE = \"x86_64\"",  
            "USER_CLASSES = \"buildstats image-mklibs image-prelink\"", 
            "PATCHRESOLVE = \"noop\"",   
            "EXTRA_IMAGE_FEATURES = \"debug-tweaks\"",
            "BB_DISKMON_DIRS = \" \\ ",  
            "   STOPTASKS,${TMPDIR},1G,100K \\ ", 
            "   STOPTASKS,${DL_DIR},1G,100K \\ ", 
            "   STOPTASKS,${SSTATE_DIR},1G,100K \\",
            "   STOPTASKS,/tmp,100M,100K \\", 
            "   ABORT,${TMPDIR},100M,1K \\",
            "   ABORT,${DL_DIR},100M,1K \\", 
            "   ABORT,${SSTATE_DIR},100M,1K \\",
            "   ABORT,/tmp,10M,1K \"" 
        ]
    },
    "tasks": { 
        "image": {
            "index": "0",
            "name": "image",
            "recipes": [
                "rpi-image" 
            ],
            "artifacts": [   
                {
                    "source": "${POKY_DEPLOY_DIR}/${MACHINE}/strix-image-${MACHINE}.rpi-sdimg"
                }
            ]
        },
        "sdk": {
            "index": "1",
            "name": "sdk",
            "disabled": "true",
            "recipes": [
                "rpi-image:do_populate_sdk"
            ],
            "artifacts": [
                {
                    "source": "${POKY_DEPLOY_DIR}/${MACHINE}/strix-sdk-${MACHINE}.sh"
                }
            ]
        }
    }
}
*/

use indexmap::IndexMap;
use serde_json::Value;
use crate::configs::{TaskConfig, BitbakeConfig, Config};
use crate::error::BError;

pub struct BuildConfig {
    version: String,
    name: String,
    description: String,
    arch: String,
    bitbake: BitbakeConfig,
    context: IndexMap<String, String>, // Optional if not set default is an empty map
    tasks: IndexMap<String, TaskConfig>, // The tasks don't have to be defined in the main build config if that is the case this will be empty
}

impl Config for BuildConfig {}

impl BuildConfig {
    fn get_tasks(data: &Value) -> Result<IndexMap<String, TaskConfig>, BError> {
        match data.get("tasks") {
            Some(value) => {
                if value.is_object() {
                    if let Some(task_map) = value.as_object() {
                        let mut tasks: IndexMap<String, TaskConfig> = IndexMap::new();
                        for (task_name, task_data) in task_map.iter() {
                            let t: TaskConfig = TaskConfig::from_value(&task_data)?;
                            tasks.insert(task_name.clone(), t);
                        }
                        return Ok(tasks);
                    }
                    return Err(BError{ code: 0, message: format!("Invalid 'tasks' format in build config")});
                } else {
                    return Err(BError{ code: 0, message: format!("Invalid 'tasks' format in build config")});
                }
            }
            None => {
                return Ok(IndexMap::new());
            }
        }
    }

    fn get_bitbake(data: &Value) -> Result<BitbakeConfig, BError> {
        match data.get("bb") {
            Some(value) => {
                BitbakeConfig::from_value(value)
            }
            None => {
                BitbakeConfig::from_str("bb: {}")
            }
        }
    }

    pub fn from_str(json_string: &str) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        let version: String = Self::get_str_value("version", &data, None)?;
        let name: String = Self::get_str_value("name", &data, None)?;
        let description: String = Self::get_str_value("description", &data, None)?;
        let arch: String = Self::get_str_value("arch", &data, None)?;
        let bitbake: BitbakeConfig = Self::get_bitbake(&data)?;
        let tasks: IndexMap<String, TaskConfig> = Self::get_tasks(&data)?;
        let context: IndexMap<String, String> = Self::get_hashmap_value("context", &data)?;
        Ok(BuildConfig {
            version,
            name,
            description,
            arch,
            bitbake,
            context,
            tasks,
        })
    }

    pub fn version(&self) -> &String {
        &self.version
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn description(&self) -> &String {
        &self.description
    }

    pub fn arch(&self) -> &String {
        &self.arch
    }

    pub fn bitbake(&self) -> &BitbakeConfig {
        &self.bitbake
    }

    pub fn tasks(&self) -> &IndexMap<String, TaskConfig> {
        &self.tasks
    }

    pub fn context(&self) -> &IndexMap<String, String> {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use crate::configs::{BuildConfig, TaskConfig};
    use crate::error::BError;
    use indexmap::IndexMap;

    fn helper_build_config_from_str(json_test_str: &str) -> BuildConfig {
        let result: Result<BuildConfig, BError> = BuildConfig::from_str(json_test_str);
        match result {
            Ok(rconfig) => {
                rconfig
            }
            Err(e) => {
                eprintln!("Error parsing build config: {}", e);
                panic!();
            } 
        }
    }

    #[test]
    fn test_build_config_simple() {
        let json_test_str = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {}
        }
        "#;
        let config = helper_build_config_from_str(json_test_str);
        assert_eq!(config.version(), "4");
        assert_eq!(config.name(), "test-name");
        assert_eq!(config.description(), "Test Description");
        assert_eq!(config.arch(), "test-arch");
    }

    #[test]
    fn test_build_config_bitbake() {
        let json_test_str = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {
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
            }
        }
        "#;
        let config = helper_build_config_from_str(json_test_str);
        assert_eq!(config.bitbake.machine(), "test-machine");
        assert_eq!(config.bitbake.distro(), "test-distro");
        assert_eq!(config.bitbake.deploy_dir(), "tmp/test/deploy");
        assert_eq!(config.bitbake.docker(), "strixos/bakery-workspace:0.68");
        assert_eq!(config.bitbake.bblayers_conf(), &vec![
            String::from("BB_LAYERS_CONF_TEST_LINE_1"),
            String::from("BB_LAYERS_CONF_TEST_LINE_2"),
            String::from("BB_LAYERS_CONF_TEST_LINE_3")
        ]);
        assert_eq!(config.bitbake.local_conf(), &vec![
            String::from("BB_LOCAL_CONF_TEST_LINE_1"),
            String::from("BB_LOCAL_CONF_TEST_LINE_2"),
            String::from("BB_LOCAL_CONF_TEST_LINE_3")
        ]);
    }

    #[test]
    fn test_build_config_optional() {
        let json_test_str = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {}
        }"#;
        let config = helper_build_config_from_str(json_test_str);
        assert_eq!(config.bitbake.machine(), "");
        assert_eq!(config.bitbake.distro(), "");
        assert_eq!(config.bitbake.deploy_dir(), "tmp/deploy/images");
        assert!(config.bitbake.bblayers_conf().is_empty());
        assert!(config.bitbake.local_conf().is_empty());
        assert!(config.tasks().is_empty());
        assert!(config.context().is_empty());
    }

    #[test]
    fn test_build_config_tasks() {
        let json_test_str = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {},
            "tasks": { 
                "task1": {
                    "index": "0",
                    "name": "task1-name",
                    "disabled": "false",
                    "type": "non-bitbake",
                    "builddir": "test/builddir",
                    "build": "build-cmd",
                    "clean": "clean-cmd",
                    "artifacts": [   
                        {
                            "source": "${BB_DEPLOY_DIR}/${MACHINE}/test-image-${MACHINE}.test-sdimg"
                        }
                    ]
                },
                "task2": {
                    "index": "1",
                    "name": "task2-name",
                    "disabled": "false",
                    "type": "bitbake",
                    "recipes": [
                        "test-image",
                        "test-image:do_populate_sdk"
                    ],
                    "artifacts": [   
                        {
                            "source": "${BB_DEPLOY_DIR}/${MACHINE}/test-image-${MACHINE}.test-sdimg"
                        }
                    ]
                }
            }
        }
        "#;
        let config = helper_build_config_from_str(json_test_str);
        let tasks: &IndexMap<String, TaskConfig> = config.tasks();
        let task: &TaskConfig = tasks.get("task1").unwrap();
        assert_eq!(task.index(), "0");
        assert_eq!(task.name(), "task1-name");
        assert_eq!(task.disabled(), "false");
        assert_eq!(task.ttype(), "non-bitbake");
        assert_eq!(task.builddir(), "test/builddir");
        assert_eq!(task.build(), "build-cmd");
        assert_eq!(task.clean(), "clean-cmd");
        let task: &TaskConfig = tasks.get("task2").unwrap();
        assert_eq!(task.index(), "1");
        assert_eq!(task.name(), "task2-name");
        assert_eq!(task.disabled(), "false");
        assert_eq!(task.ttype(), "bitbake");
        assert_eq!(task.recipes(), &vec![String::from("test-image"), String::from("test-image:do_populate_sdk")]);
    }

    #[test]
    fn test_build_config_context() {
        let json_test_str = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "context": [
                "CONTEXT_1=context1",
                "CONTEXT_2=context2",
                "CONTEXT_3=context3"
            ],
            "bb": {}
        }
        "#;
        let config = helper_build_config_from_str(json_test_str);
        assert!(!config.context().is_empty());
        let mut i = 1;
        for (key, value) in config.context().iter() {
            println!("Key: {}, Value: {}", key, value);
            assert_eq!(key, &format!("CONTEXT_{}", i));
            assert_eq!(value, &format!("context{}", i));
            i += 1;
        }
    }
}