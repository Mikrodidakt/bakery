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
use std::{collections::HashMap, hash::Hash};
use serde_json::Value;
use crate::error::BError;
use crate::configs::TaskConfig;

pub trait Config {
    fn get_str_value(name: &str, data: &Value, default: Option<String>) -> Result<String, BError> {
        match data.get(name) {
            Some(value) => {
                if value.is_string() {
                    Ok(value.as_str().unwrap().to_string())
                } else {
                    return Err(BError{ code: 0, message: format!("Failed to read '{}' is not a string", name)});
                }
            }
            None => {
                match default {
                    Some(default_value) => Ok(default_value),
                    None => Err(BError {
                        code: 0,
                        message: format!("Failed to read '{}'", name),
                    }),
                }
            }
        }
    }

    fn get_array_value(name: &str, data: &Value, default: Option<Vec<String>>) -> Result<Vec<String>, BError> {
        match data.get(name) {
            Some(array_value) => {
                if array_value.is_array() {
                    return Ok(array_value
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_owned())
                    .collect());
                } else {
                    return Err(BError{ code: 0, message: format!("Failed to read '{}' is not a string", name)});
                }
            }
            None => {
                match default {
                    Some(default_value) => Ok(default_value),
                    None => Err(BError {
                        code: 0,
                        message: format!("Failed to read '{}'", name),
                    }),
                }
            }
        }
    }

    fn get_hashmap_value(name: &str, data: &Value) -> Result<HashMap<String, String>, BError> {
        match data.get(name) {
            Some(array_value) => {
                if array_value.is_array() {
                    let mut hashmap: HashMap<String, String> = HashMap::new();
                    for value in array_value.as_array().unwrap().iter() {
                        let pair: String = value.to_string();
                        let parts: Vec<&str> = pair.splitn(2, '=').collect();
                        
                        if parts.len() == 2 {
                            let key = parts[0].to_string();
                            let value = parts[1].to_string();
                            hashmap.insert(key, value);
                        }
                    }
                    Ok(hashmap)
                } else {
                    return Err(BError{ code: 0, message: format!("Failed to read '{}' is not a string", name)});
                }
            }
            None => {
                //return Err(BError{ code: 0, message: format!("Failed to read '{}'", name)});
                return Ok(HashMap::new());
            }
        }
    }

    fn get_value<'a>(name: &str, data: &'a Value) -> Result<&'a Value, BError> {
        match data.get(name) {
            Some(value) => Ok(value),
            None => Err(BError {
                code: 0,
                message: format!("Failed to read '{}'", name),
            }),
        }
    }

    fn parse(json_string: &str) -> Result<Value, BError> {
        match serde_json::from_str(json_string) {
            Ok(data) => {
                Ok(data) 
            },
            Err(err) => {
                let error_message = format!("Failed to parse JSON: {}", err);
                Err(BError { code: 1, message: error_message })
            }
        }
    }
}

pub struct BuildConfig {
    version: String,
    name: String,
    description: String,
    arch: String,
    machine: String, // Optional but if there is a task with type bitbake defined it might fail
    distro: String, // Optional but if there is a task with type bitbake defined it might fail
    deploydir: String, // Optional if not set the default deploy dir will be used builds/tmp/deploydir
    context: HashMap<String, String>, // Optional if not set default is an empty map
    bb_layers_conf: Vec<String>, // Optional but if there is a task with type bitbake defined it will fail without a bblayers.conf
    bb_local_conf: Vec<String>, // Optional but if there is a task with type bitbake defined it will fail without a local.conf
    tasks: HashMap<String, TaskConfig>, // The tasks don't have to be defined in the main build config if that is the case this will be empty
}

impl Config for BuildConfig {}

impl BuildConfig {
    fn get_tasks(data: &Value) -> Result<HashMap<String, TaskConfig>, BError> {
        match data.get("tasks") {
            Some(value) => {
                if value.is_object() {
                    if let Some(task_map) = value.as_object() {
                        let mut tasks: HashMap<String, TaskConfig> = HashMap::new();
                        for (task_name, task_data) in task_map.iter() {
                            let t: TaskConfig = TaskConfig::from_value(&task_data)?;
                            tasks.insert(task_name.clone(), t);
                        }
                        return Ok(tasks);
                    }
                    return Err(BError{ code: 0, message: format!("Invalid Tasks format in build config")});
                } else {
                    return Err(BError{ code: 0, message: format!("Invalid Tasks format in build config")});
                }
            }
            None => {
                return Ok(HashMap::new());
            }
        }
    }

    pub fn from_str(json_string: &str) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        let version: String = Self::get_str_value("version", &data, None)?;
        let name: String = Self::get_str_value("name", &data, None)?;
        let description: String = Self::get_str_value("description", &data, None)?;
        let arch: String = Self::get_str_value("arch", &data, None)?;
        let bb_data: &Value = Self::get_value("bb", &data)?;
        let machine: String = Self::get_str_value("machine", &bb_data, Some(String::from("")))?;
        let distro: String = Self::get_str_value("distro", &bb_data, Some(String::from("")))?;
        let deploydir: String = Self::get_str_value("deploydir", &bb_data, Some(String::from("tmp/deploy/images")))?;
        let bb_layers_conf: Vec<String> = Self::get_array_value("bblayersconf", &bb_data, Some(vec![]))?;
        let bb_local_conf: Vec<String> = Self::get_array_value("localconf", &bb_data, Some(vec![]))?;
        let tasks: HashMap<String, TaskConfig> = Self::get_tasks(&data)?;
        let context: HashMap<String, String> = Self::get_hashmap_value("context", &data)?;
        Ok(BuildConfig {
            version,
            name,
            description,
            arch,
            machine,
            distro,
            deploydir,
            context,
            bb_layers_conf,
            bb_local_conf,
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

    pub fn machine(&self) -> &String {
        &self.machine
    }

    pub fn distro(&self) -> &String {
        &self.distro
    }

    pub fn deploydir(&self) -> &String {
        &self.deploydir
    }

    pub fn bblayersconf(&self) -> &Vec<String> {
        &self.bb_layers_conf
    }

    pub fn localconf(&self) -> &Vec<String> {
        &self.bb_local_conf
    }

    pub fn tasks(&self) -> &HashMap<String, TaskConfig> {
        &self.tasks
    }

    pub fn context(&self) -> &HashMap<String, String> {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use crate::configs::{BuildConfig, TaskConfig};
    use crate::error::BError;
    use std::collections::HashMap;

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
        assert_eq!(config.machine(), "test-machine");
        assert_eq!(config.distro(), "test-distro");
        assert_eq!(config.deploydir(), "tmp/test/deploy");
        assert_eq!(config.bblayersconf(), &vec![
            String::from("BB_LAYERS_CONF_TEST_LINE_1"),
            String::from("BB_LAYERS_CONF_TEST_LINE_2"),
            String::from("BB_LAYERS_CONF_TEST_LINE_3")
        ]);
        assert_eq!(config.localconf(), &vec![
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
        }
        "#;
        let config = helper_build_config_from_str(json_test_str);
        assert_eq!(config.machine(), "");
        assert_eq!(config.distro(), "");
        assert_eq!(config.deploydir(), "tmp/deploy/images");
        assert!(config.bblayersconf().is_empty());
        assert!(config.localconf().is_empty());
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
        let tasks: &HashMap<String, TaskConfig> = config.tasks();
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
}