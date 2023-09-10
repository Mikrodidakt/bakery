use std::{collections::HashMap, hash::Hash};

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
use serde_json::Value;
use crate::error::BError;

trait Config {
    fn get_str_value(name: &str, data: &Value, default: Option<String>) -> Result<String, BError> {
        match data.get(name) {
            Some(value) => {
                if value.is_string() {
                    Ok(value.to_string())
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

    fn get_array_value(name: &str, data: &Value) -> Result<Vec<String>, BError> {
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
                return Err(BError{ code: 0, message: format!("Failed to read '{}'", name)});
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
                return Err(BError{ code: 0, message: format!("Failed to read '{}'", name)});
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
    machine: String,
    distro: String,
    deploydir: String, // Optional if not set the default deploy dir will be used builds/tmp/deploydir
    bb_layers_conf: Vec<String>,
    bb_local_conf: Vec<String>,
    tasks: Vec<Task>, // The tasks don't have to be defined in the main build config if that is the case this will be empty
}

pub struct Task {
    index: String,
    name: String,
    ttype: String, // Optional if not set for the task the default type 'bitbake' is used
    disabled: bool, // Optional if not set for the task the default value 'false' is used
    builddir: String,
    build: String,
    clean: String,
    recipes: Vec<String>, // The list of recipes will be empty if the type for the task is 'non-bitbake'
    artifacts: Value, // For some tasks there might not be any artifacts to collect then this will be empty
    context: HashMap<String, String>, // The context is optional
}

impl Config for BuildConfig {}

impl BuildConfig {
    fn parse_task(data: &Value) -> Result<Task, BError> {
        let index: String = Self::get_str_value("index", &data, None)?;
        let name: String = Self::get_str_value("name", &data, None)?;
        let ttype: String = Self::get_str_value("type", &data, Some(String::from("bitbake")))?;
        let disabled: String = Self::get_str_value("disabled", &data, Some(String::from("false")))?;
        let builddir: String = Self::get_str_value("builddir", &data, None)?;
        let build: String = Self::get_str_value("build", &data, None)?;
        let clean: String = Self::get_str_value("clean", &data, None)?;
        let recipes: Vec<String> = Self::get_array_value("recipes", &data)?;
        let artifacts: &Value = Self::get_value("artifacts", &data)?;
        let context: HashMap<String, String> = Self::get_hashmap_value("context", &data)?;
        Ok(Task {
            index,
            name,
            ttype,
            disabled: disabled.parse().unwrap(),
            builddir,
            build,
            clean,
            recipes,
            artifacts: artifacts.clone(),
            context,
        })
    }

    fn get_tasks(data: &Value) -> Result<Vec<Task>, BError> {
        match data.get("tasks") {
            Some(value) => {
                if value.is_array() {
                    let mut tasks: Vec<Task> = Vec::new();
                    for task in value.as_array().unwrap().iter() {
                        let t: Task = Self::parse_task(&task)?;
                        tasks.push(t);
                    }
                    Ok(tasks)
                } else {
                    return Err(BError{ code: 0, message: format!("Invalid Tasks format in build config")});
                }
            }
            None => {
                // TODO a build config might not have any tasks defined since the might be defined in one
                // of the build configs included by the include node in the main build config. We should
                // make sure to support that and not return an error instead we should return an empty
                // vector or come up with some other solution for this scenario.
                return Err(BError{ code: 0, message: format!("Failed to read tasks")});
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
        let machine: String = Self::get_str_value("machine", &bb_data, None)?;
        let distro: String = Self::get_str_value("distro", &bb_data, None)?;
        let deploydir: String = Self::get_str_value("deploydir", &bb_data, Some(String::from("tmp/deploy/images")))?;
        let bb_layers_conf: Vec<String> = Self::get_array_value("bblayersconf", &bb_data)?;
        let bb_local_conf: Vec<String> = Self::get_array_value("localconf", &bb_data)?;
        let tasks: Vec<Task> = Self::get_tasks(&data)?;
        let context: HashMap<String, String> = Self::get_hashmap_value("context", &data)?;
        Ok(BuildConfig {
            version,
            name,
            description,
            arch,
            machine,
            distro,
            deploydir,
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

    pub fn tasks(&self) -> &Vec<Task> {
        &self.tasks
    } 
}