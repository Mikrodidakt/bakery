use indexmap::{indexmap, IndexMap};
use serde_json::Value;
use std::path::PathBuf;

use crate::configs::Context;
use crate::error::BError;
use crate::workspace::{WsArtifactsHandler, WsSettingsHandler, WsTaskHandler};
use crate::fs::JsonFileReader;
use crate::configs::Config;

pub struct WsConfigData {
    version: String,
    name: String,
}

impl Config for WsConfigData {}

impl WsConfigData {
    pub fn from_str(json_string: &str) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        Self::from_value(&data)
    }

    pub fn from_value(data: &Value) -> Result<Self, BError> {
        let version: String = Self::get_str_value("version", &data, None)?;
        let name: String = Self::get_str_value("name", &data, Some(String::from("NA")))?;
        Ok(WsConfigData {
            version,
            name,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

pub struct WsProductData {
    name: String,
    arch: String,
    description: String,
}

impl Config for WsProductData {}

impl WsProductData {
    pub fn from_str(json_string: &str) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        Self::from_value(&data)
    }

    pub fn from_value(data: &Value) -> Result<Self, BError> {
        let name: String = Self::get_str_value("name", &data, Some(String::from("NA")))?;
        let description: String = Self::get_str_value("description", &data, Some(String::from("NA")))?;
        let arch: String = Self::get_str_value("arch", &data, Some(String::from("NA")))?;
        Ok(WsProductData {
            name,
            arch,
            description,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arch(&self) -> &str {
        &self.arch
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

pub struct WsBitbakeData {
    product: String, // This is required and is not part of the bitbake segment but is used when putting the bitbake data together
    arch: String, // This is required and is not part of the bitbake segment but is used when putting the bitbake data together
    machine: String, // Optional but if there is a task with type bitbake defined it might fail
    distro: String, // Optional but if there is a task with type bitbake defined it might fail
    deploy_dir: String, // Optional if not set the default deploy dir will be used builds/tmp/deploydir
    docker: String, // Optional if nothing is set the bitbake task will be executed inside the bakery container. Default is an empty string
    bblayers_conf: Vec<String>, // Optional but if there is a task with type bitbake defined it will fail without a bblayers.conf
    local_conf: Vec<String>, // Optional but if there is a task with type bitbake defined it will fail without a local.conf
    settings: WsSettingsHandler,
}

impl Config for WsBitbakeData {
}

impl WsBitbakeData {
    pub fn from_str(json_string: &str, settings: &WsSettingsHandler) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        Self::from_value(&data, settings)
    }

    pub fn from_value(data: &Value, settings: &WsSettingsHandler) -> Result<Self, BError> {
        let mut bb_data: &Value = data;
        match data.get("bb") {
            Some(value) => {
                bb_data = value;
            }
            None => {}
        }
        // Not part of the bitbake segment in the build config but is required when putting the bitbake
        // build data together
        let product: String = Self::get_str_value("name", data, Some(String::from("NA")))?;
        // Not part of the bitbake segment in the build config but is required when putting the bitbake
        // build data together
        let arch: String = Self::get_str_value("arch", data, Some(String::from("NA")))?;
        let machine: String = Self::get_str_value("machine", bb_data, Some(String::from("NA")))?;
        let distro: String = Self::get_str_value("distro", bb_data, Some(String::from("NA")))?;
        let docker: String = Self::get_str_value("docker", bb_data, Some(String::from("NA")))?;
        let deploy_dir: String = Self::get_str_value("deploydir", bb_data, Some(String::from("tmp/deploy/images")))?;
        let bblayers_conf: Vec<String> = Self::get_array_value("bblayersconf", bb_data, Some(vec![]))?;
        let local_conf: Vec<String> = Self::get_array_value("localconf", bb_data, Some(vec![]))?;

        Ok(WsBitbakeData {
            product,
            arch,
            machine,
            distro,
            docker,
            deploy_dir,
            bblayers_conf,
            local_conf,
            settings: settings.clone(),
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

    pub fn bblayers_conf(&self) -> &Vec<String> {
        &self.bblayers_conf
    }

    pub fn local_conf(&self) -> Vec<String> {
        let mut local_conf: Vec<String> = self.local_conf.clone();
        local_conf.push(format!("MACHINE ?= {}", self.machine()));
        // TODO: we need to handle VARIANT correctly but this is good enough for now
        local_conf.push(format!("VARIANT ?= {}", "dev".to_string()));
        // TODO: we should define a method product_name() call that instead
        local_conf.push(format!("PRODUCT_NAME ?= {}", self.product));
        local_conf.push(format!("DISTRO ?= {}", self.distro));
        local_conf.push(format!("SSTATE_DIR ?= {}", self.sstate_dir().to_str().unwrap()));
        local_conf.push(format!("DL_DIR ?= {}", self.dl_dir().to_str().unwrap()));
        //local_conf.push(format!("PLATFORM_VERSION ?= {}", self.platform_version()));
        //local_conf.push(format!("BUILD_NUMBER ?= {}", self.build_number()));
        //local_conf.push(format!("BUILD_SHA ?= {}", self.build_sha()));
        //local_conf.push(format!("RELASE_BUILD ?= {}", self.release_build()));
        //local_conf.push(format!("BUILD_VARIANT ?= {}", self.build_variant()));
        local_conf
    }

    pub fn machine(&self) -> &str {
        &self.machine
    }

    pub fn distro(&self) -> &str {
        &self.distro
    }

    pub fn build_dir(&self) -> PathBuf {
        self.settings.builds_dir().clone().join(PathBuf::from(self.product.clone()))
    }

    pub fn docker_image(&self) -> &str {
        &self.docker
    }

    pub fn build_config_dir(&self) -> PathBuf {
        self.build_dir().join("conf".to_string())
    }

    pub fn local_conf_path(&self) -> PathBuf {
        self.build_config_dir().join("local.conf".to_string())
    }

    pub fn bblayers_conf_path(&self) -> PathBuf {
        self.build_config_dir().join("bblayers.conf")
    }

    pub fn deploy_dir(&self) -> PathBuf {
        self.build_dir().join(PathBuf::from(self.deploy_dir.clone()))
    }    

    pub fn sstate_dir(&self) -> PathBuf {
        self.settings.cache_dir().clone().join(&self.arch).join("sstate-cache".to_string())
    }

    pub fn dl_dir(&self) -> PathBuf {
        self.settings.cache_dir().clone().join("download".to_string())
    }        

    pub fn poky_dir(&self) -> PathBuf {
        // TODO: not sure about this we should not lock the bakery into using poky
        // we only need this to be able to determine where to find the OE init file.
        // I think the solution is to add a entry in the build config file in the bb-node
        // where you can specify a path for the init file to source. The default could be
        // layers/poky/oe-init-build-env. Potentially we should also add an entry in the
        // Workspace settings file where you can specify the layers directory
        self.settings.work_dir().clone().join("layers".to_string()).join("poky".to_string())
    }

    pub fn oe_init_file(&self) -> PathBuf {
        // TODO: we should probably setup an option to configure what OE init script
        // to source to setup the env.
        self.poky_dir().join("oe-init-build-env")
    }
}

pub struct WsContextData {
    context: Context,
}

impl Config for WsContextData {}

impl WsContextData {
    pub fn from_str(json_string: &str) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        Self::from_value(&data)
    }

    pub fn from_value(data: &Value) -> Result<Self, BError> {
        let variables: IndexMap<String, String> = Self::get_hashmap_value("context", &data)?;
        let ctx_default_variables: IndexMap<String, String> = indexmap! {
            "MACHINE".to_string() => "NA".to_string(),
            "ARCH".to_string() => "NA".to_string(),
            "DISTRO".to_string() => "NA".to_string(),
            "VARIANT".to_string() => "NA".to_string(),
            "PRODUCT_NAME".to_string() => "NA".to_string(),
            "BB_BUILD_DIR".to_string() => "".to_string(),
            "BB_DEPLOY_DIR".to_string() => "".to_string(),
            "ARTIFACTS_DIR".to_string() => "".to_string(),
            "BUILDS_DIR".to_string() => "".to_string(),
            "WORK_DIR".to_string() => "".to_string(),
            "PLATFORM_VERSION".to_string() => "0.0.0".to_string(),
            "BUILD_NUMBER".to_string() => "0".to_string(),
            "PLATFORM_RELEASE".to_string() => "0.0.0-0".to_string(), // We should combine the PLATFORM_VERSION with the BUILD_NUMBER
            "BUILD_SHA".to_string() => "dev".to_string(), // If no git sha is specified and it is built locally then this is the default
            "RELEASE_BUILD".to_string() => "0".to_string(),
            "BUILD_VARIANT".to_string() => "dev".to_string(), // The variant can be dev, release and manufacturing
            "ARCHIVER".to_string() => "0".to_string(),
            "DEBUG_SYMBOLS".to_string() => "0".to_string(), // This can be used if you need to collect debug symbols from a build and have specific task defined for it
        };
        let mut ctx: Context = Context::new(&ctx_default_variables);
        ctx.update(&variables);
        Ok(WsContextData {
            context: ctx,
        })
    }

    /*
    pub fn variables(&self) -> &IndexMap<String, String> {
        &self.variables
    }
    */
    pub fn ctx(&self) -> &Context {
        &self.context
    }

    pub fn update(&mut self, variables: &IndexMap<String, String>) {
        self.context.update(variables);
    }

    pub fn get_ctx_path(&self, key: &str) -> PathBuf {
        PathBuf::from(self.get_ctx_value(key))
    }

    pub fn get_ctx_value(&self, key: &str) -> String {
        self.context.value(key)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum TType {
    Bitbake,
    NonBitbake,
}

pub struct WsTaskData {
    index: String,
    name: String,
    ttype: TType, // Optional if not set for the task the default type 'bitbake' is used
    disabled: String, // Optional if not set for the task the default value 'false' is used
    build_dir: PathBuf,
    build: String,
    docker: String,
    condition: String,
    clean: String,
    recipes: Vec<String>, // The list of recipes will be empty if the type for the task is 'non-bitbake'
}

impl Config for WsTaskData {
}

impl WsTaskData {
    fn determine_build_dir(ttype: TType, task_build_dir: &str, data: &WsBuildData) -> PathBuf {
        if ttype == TType::Bitbake {
            if task_build_dir.is_empty() {
                return data.bitbake().build_dir()
            }
        }

        data.settings().work_dir().join(PathBuf::from(task_build_dir))
    }

    pub fn from_str(json_string: &str, build_data: &WsBuildData) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        Self::from_value(&data, build_data)
    }

    pub fn from_value(data: &Value, build_data: &WsBuildData) -> Result<Self, BError> {
        let index: String = Self::get_str_value("index", &data, None)?;
        let name: String = Self::get_str_value("name", &data, None)?;
        let ttype: String = Self::get_str_value("type", &data, Some(String::from("bitbake")))?;
        let disabled: String = Self::get_str_value("disabled", &data, Some(String::from("false")))?;
        let mut build_dir: String = Self::get_str_value("builddir", &data, Some(String::from("")))?;
        let docker: String = Self::get_str_value("docker", data, Some(String::from("")))?;
        let condition: String = Self::get_str_value("condition", data, Some(String::from("true")))?;
        let build: String = Self::get_str_value("build", &data, Some(String::from("")))?;
        let clean: String = Self::get_str_value("clean", &data, Some(String::from("")))?;
        let recipes: Vec<String> = Self::get_array_value("recipes", &data, Some(vec![]))?;

        let enum_ttype: TType;
        match ttype.as_str() {
            "bitbake" => {
                enum_ttype = TType::Bitbake;
            },
            "non-bitbake" => {
                enum_ttype = TType::NonBitbake;
            },
            _ => {
                return Err(BError::ParseTasksError(format!("Invalid type '{}'", ttype)));
            },
        }

        build_dir = build_data.context().ctx().expand_str(&build_dir);
        let task_build_dir: PathBuf = Self::determine_build_dir(enum_ttype.clone(), &build_dir, build_data);

        // if the task type is bitbake then at least one recipe is required
        if recipes.is_empty() && ttype == "bitbake" {
            return Err(BError::ParseTasksError(format!("The 'bitbake' type requires at least one entry in 'recipes'")));
        }

        Ok(WsTaskData {
            index,
            name,
            ttype: enum_ttype,
            disabled,
            docker,
            condition,
            build_dir: task_build_dir,
            build,
            clean,
            recipes,
        })
    }
    
    pub fn expand_ctx(&mut self, ctx: &Context) {
        self.build_dir = ctx.expand_path(&self.build_dir);
        self.build = ctx.expand_str(&self.build);
        self.clean = ctx.expand_str(&self.clean);
        self.condition = ctx.expand_str(&self.condition);
        self.recipes.iter_mut().for_each(|r: &mut String| *r = ctx.expand_str(r));
    }

    pub fn index(&self) -> u32 {
        1
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ttype(&self) -> &TType {
        &self.ttype
    }

    pub fn disabled(&self) -> bool {
        if self.disabled == "true" {
            return true;
        }
        return false;
    }

    pub fn docker_image(&self) -> &str {
        &self.docker
    }

    pub fn condition(&self) -> bool {
        let condition: &str = &self.condition;

        if condition.is_empty() {
            return true;
        }

        match condition {
            "1" | "yes" | "y" | "Y" | "true" | "YES" | "TRUE" | "True" | "Yes" => return true,
            _ => return false,
        }
    }

    pub fn build_dir(&self) -> &PathBuf {
        &self.build_dir
    }

    pub fn build_cmd(&self) -> &str {
        &self.build
    }

    pub fn clean_cmd(&self) -> &str {
        &self.clean
    }

    pub fn recipes(&self) -> &Vec<String> {
        &self.recipes
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum AType {
    File,
    Directory,
    Archive,
    Manifest,
}

//TODO: we should consider using IndexSet instead of vector to make sure we
// keep the order from the json file
pub struct WsArtifactData {
    pub atype: AType, // Optional if not set for the task the default type 'file' is used
    pub name: String, // The name can be a name for a directory, archive, file or manifest
    pub source: PathBuf, // The source is only used if the type is file 
    pub dest: PathBuf, // The dest is optional
    pub manifest: String, // The manifest content will be a json string that can be put in a file. The manifest can then be used by the CI to collect information from the build
}

impl Config for WsArtifactData {
}

impl WsArtifactData {
    pub fn from_str(json_string: &str, task_build_dir: &PathBuf, build_data: &WsBuildData) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        Self::from_value(&data, task_build_dir, build_data)
    }

    pub fn from_value(data: &Value, task_build_dir: &PathBuf, build_data: &WsBuildData) -> Result<Self, BError> {
        let ttype: String = Self::get_str_value("type", &data, Some(String::from("file")))?;
        let name: String = Self::get_str_value("name", &data, Some(String::from("")))?;
        let source_str: String = Self::get_str_value("source", &data, Some(String::from("")))?;
        let dest_str: String = Self::get_str_value("dest", &data, Some(String::from("")))?;
        let manifest: String = Self::get_str_manifest("content", &data, Some(String::from("{}")))?;
        if ttype != "file" && ttype != "directory" && ttype != "archive" && ttype != "manifest" {
            return Err(BError::ParseArtifactsError(format!("Invalid type '{}'", ttype)));
        }
        if ttype == "file" && source_str.is_empty() {
            return Err(BError::ParseArtifactsError(format!("The 'file' type requires a 'source'")));
        }
        if ttype == "directory" && name.is_empty() {
            return Err(BError::ParseArtifactsError(format!("The 'directory' type requires a 'name'")));
        }
        if ttype == "archive" && name.is_empty() {
            return Err(BError::ParseArtifactsError(format!("The 'archive' type requires a 'name'")));
        }
        if ttype == "manifest" && name.is_empty() {
            return Err(BError::ParseArtifactsError(format!("The 'manifest' type requires a 'name'")));
        }

        let enum_ttype: AType;
        match ttype.as_str() {
            "file" => {
                enum_ttype = AType::File;
            },
            "directory" => {
                enum_ttype = AType::Directory;
            },
            "archive" => {
                enum_ttype = AType::Archive;
            },
            "manifest" => {
                enum_ttype = AType::Manifest;
            },
            _ => {
                return Err(BError::ParseArtifactsError(format!("Invalid type '{}'", ttype)));
            },
        }

        let source: PathBuf = task_build_dir.clone().join(build_data.context().ctx().expand_str(&source_str));
        let dest: PathBuf = build_data.settings().artifacts_dir().clone().join(build_data.context().ctx().expand_str(&dest_str));

        Ok(WsArtifactData {
            name,
            atype: enum_ttype,
            source,
            dest,
            manifest,
        })
    }

    pub fn expand_ctx(&mut self, ctx: &Context) {
        match self.atype {
            AType::File => {
                self.name = ctx.expand_str(&self.name);
                self.source = ctx.expand_path(&self.source);
                self.dest = ctx.expand_path(&self.dest);
            },
            AType::Directory => {
                self.name = ctx.expand_str(&self.name);
            },
            AType::Archive => {
                self.name = ctx.expand_str(&self.name);
            },
            AType::Manifest => {
                self.name = ctx.expand_str(&self.name);
                self.manifest = ctx.expand_str(&self.manifest);
            },
            _ => {
                panic!("Invalid 'artifact' format in build config. Invalid type '{:?}'", self.atype);
            },
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn atype(&self) -> &AType {
        &self.atype
    }

    pub fn source(&self) -> &PathBuf {
        &self.source
    }

    pub fn dest(&self) -> &PathBuf {
        &self.dest
    }

    pub fn manifest(&self) -> &str {
        &self.manifest
    }
}

pub struct WsBuildData {
    data: Value,
    config: WsConfigData,
    product: WsProductData,
    bitbake: WsBitbakeData,
    context: WsContextData,
    settings: WsSettingsHandler,
}

impl WsBuildData {
    fn get_task(&self, data: &Value) -> Result<WsTaskHandler, BError> {
        let mut task: WsTaskHandler = WsTaskHandler::new(data, &self)?;
        task.expand_ctx(self.context().ctx());
        Ok(task)
    }

    fn get_artifact(&self, data: &Value, task_build_dir: &PathBuf) -> Result<WsArtifactsHandler, BError> {
        let mut artifact: WsArtifactsHandler = WsArtifactsHandler::new(data, task_build_dir, &self)?;
        artifact.expand_ctx(self.context().ctx());
        Ok(artifact)
    }

    pub fn from_str(json_config: &str, settings: &WsSettingsHandler) -> Result<Self, BError> {
        let data: Value = JsonFileReader::parse(json_config)?;
        Self::new(&data, settings)
    }

    pub fn new(data: &Value, settings: &WsSettingsHandler) -> Result<Self, BError> {
        // Parse the individual segments of the build config

        // The config segments contains data related with the actual config
        // like version and configuration name which normally is the same
        // as the product name but that might change in the future
        let config: WsConfigData = WsConfigData::from_value(data)?;
        // The product segments contains product specific data such as
        // product name and arch
        let product: WsProductData = WsProductData::from_value(data)?;
        // The bitbake segment contains all the bitbake related data
        // needed when executing a bitbake task defined in the build
        // config
        let mut bitbake: WsBitbakeData = WsBitbakeData::from_value(data, settings)?;
        // The context segment contains all the context variables used
        // by other parts of the build config
        let mut context: WsContextData = WsContextData::from_value(data)?;

        // Setup context with "built-in" variables that will always
        // be available
        let ctx_built_in_variables: IndexMap<String, String> = indexmap! {
            "MACHINE".to_string() => bitbake.machine.clone(),
            "ARCH".to_string() => product.arch.clone(),
            "DISTRO".to_string() => bitbake.distro.clone(),
            "PRODUCT_NAME".to_string() => product.name.clone(),
            "ARTIFACTS_DIR".to_string() => settings.artifacts_dir().to_string_lossy().to_string(),
            "BUILDS_DIR".to_string() => settings.builds_dir().to_string_lossy().to_string(),
            "WORK_DIR".to_string() => settings.work_dir().to_string_lossy().to_string(),
        };
        context.update(&ctx_built_in_variables);
        // Expand all the context variables in the build config
        bitbake.expand_ctx(context.ctx());
        // Update the "built-in" bitbake paths in the context variables
        let bb_build_dir: PathBuf = settings.builds_dir().clone().join(PathBuf::from(product.name.clone()));
        let bb_deploy_dir: PathBuf = bb_build_dir.clone().join(PathBuf::from(bitbake.deploy_dir.clone()));
        let ctx_bitbake_variables: IndexMap<String, String> = indexmap! {
            "BB_BUILD_DIR".to_string() => bb_build_dir.to_string_lossy().to_string(),
            "BB_DEPLOY_DIR".to_string() => bb_deploy_dir.to_string_lossy().to_string(),
        };
        context.update(&ctx_bitbake_variables);

        Ok(WsBuildData {
            data: data.to_owned(),
            config,
            product,
            bitbake,
            context,
            settings: settings.clone(), // for now lets clone it
        })
    }

    pub fn get_artifacts(&self, data: &Value, task_build_dir: &PathBuf) -> Result<Vec<WsArtifactsHandler>, BError> {
        match data.get("artifacts") {
            Some(value) => {
                if value.is_array() {
                    if let Some(artifact_vec) = value.as_array() {
                        let mut artifacts: Vec<WsArtifactsHandler> = Vec::new();
                        for artifact_data in artifact_vec.iter() {
                            let artifact: WsArtifactsHandler =
                                self.get_artifact(artifact_data, task_build_dir)?;
                            artifacts.push(artifact);
                        }
                        return Ok(artifacts);
                    }
                    return Err(BError::ParseArtifactsError("Invalid 'artifacts' node in build config".to_string()));
                } else {
                    return Err(BError::ParseArtifactsError("No 'artifacts' array node found in build config".to_string()));
                }
            }
            None => {
                return Ok(Vec::new());
            }
        }
    }

    pub fn get_tasks(&self, data: &Value) -> Result<IndexMap<String, WsTaskHandler>, BError> {
        match data.get("tasks") {
            Some(value) => {
                if value.is_object() {
                    if let Some(task_map) = value.as_object() {
                        let mut tasks: IndexMap<String, WsTaskHandler> = IndexMap::new();
                        for (name, data) in task_map.iter() {
                            let task: WsTaskHandler = self.get_task(data)?;
                            tasks.insert(name.clone(), task);
                        }
                        return Ok(tasks);
                    }
                    return Err(BError::ParseTasksError("Invalid 'task' format in build config".to_string()));
                } else {
                    return Err(BError::ParseTasksError("No 'tasks' node found in build config".to_string()));
                }
            }
            None => {
                return Ok(IndexMap::new());
            }
        }
    }

    pub fn name(&self) -> &str {
        self.config.name()
    }

    pub fn version(&self) -> &str {
        self.config.version()
    }

    pub fn settings(&self) -> &WsSettingsHandler {
        &self.settings
    }

    pub fn context(&self) -> &WsContextData {
        &self.context
    }

    pub fn product(&self) -> &WsProductData {
        &self.product
    }

    pub fn bitbake(&self) -> &WsBitbakeData {
        &self.bitbake
    }
}

#[cfg(test)]
mod tests {
    use indexmap::{indexmap, IndexMap};
    use serde_json::Value;
    use std::path::PathBuf;

    use crate::error::BError;
    use crate::fs::JsonFileReader;
    use crate::workspace::{WsArtifactsHandler, WsContextData, WsSettingsHandler, WsBuildData, WsConfigData, WsProductData, WsTaskHandler, WsBitbakeData, AType};
    use crate::helper::Helper;

    #[test]
    fn test_ws_config_data_default() {
        let json_default_build_config = r#"
        {                                                                                                                   
            "version": "4"
        }"#;
        let data: WsConfigData = WsConfigData::from_str(json_default_build_config).expect("Failed to parse config data");
        assert_eq!(data.version(), "4");
        assert_eq!(data.name(), "NA");
    }

    #[test]
    fn test_ws_config_data() {
        let json_default_build_config = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name"
        }"#;
        let data: WsConfigData = WsConfigData::from_str(json_default_build_config).expect("Failed to parse config data");
        assert_eq!(data.version(), "4");
        assert_eq!(data.name(), "test-name");
    }

    #[test]
    fn test_ws_product_data_default() {
        let json_default_build_config = r#"
        {                                                                                                                   
            "version": "4"
        }"#;
        let data: WsProductData = WsProductData::from_str(json_default_build_config).expect("Failed to parse product data");
        assert_eq!(data.name(), "NA");
        assert_eq!(data.description(), "NA");
        assert_eq!(data.arch(), "NA");
    }

    #[test]
    fn test_ws_product_data() {
        let json_default_build_config = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "test description",
            "arch": "test-arch"
        }"#;
        let data: WsProductData = WsProductData::from_str(json_default_build_config).expect("Failed to parse product data");
        assert_eq!(data.name(), "test-name");
        assert_eq!(data.description(), "test description");
        assert_eq!(data.arch(), "test-arch");
    }

    #[test]
    fn test_ws_bitbake_data_default() {
        let json_settings: &str = r#"
        {
            "version": "4"
        }"#;
        let json_build_config = r#"
        {                                                                                                                   
            "version": "4"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_settings).expect("Failed to parse settings");
        let data: WsBitbakeData = WsBitbakeData::from_str(json_build_config, &settings).expect("Failed to parse product data");
        assert_eq!(data.machine(), "NA");
        assert_eq!(data.distro(), "NA");
        assert_eq!(data.docker_image(), "NA");
        assert_eq!(data.build_dir(), PathBuf::from(String::from("/workspace/builds/NA")));
        assert_eq!(data.build_config_dir(), PathBuf::from(String::from("/workspace/builds/NA/conf")));
        assert_eq!(data.deploy_dir(), PathBuf::from(String::from("/workspace/builds/NA/tmp/deploy/images")));
        assert_eq!(data.bblayers_conf_path(), PathBuf::from(String::from("/workspace/builds/NA/conf/bblayers.conf")));
        assert_eq!(data.local_conf_path(), PathBuf::from(String::from("/workspace/builds/NA/conf/local.conf")));
        assert!(data.bblayers_conf().is_empty());
        assert!(!data.local_conf().is_empty());
        assert_eq!(data.sstate_dir(), PathBuf::from(String::from("/workspace/.cache/NA/sstate-cache")));
        assert_eq!(data.dl_dir(), PathBuf::from(String::from("/workspace/.cache/download")));
        assert_eq!(data.poky_dir(), PathBuf::from(String::from("/workspace/layers/poky")));
        assert_eq!(data.oe_init_file(), PathBuf::from(String::from("/workspace/layers/poky/oe-init-build-env")));
    }

    #[test]
    fn test_ws_bitbake_data() {
        let json_settings: &str = r#"
        {
            "version": "4"
        }"#;
        let json_build_config = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "arch": "test-arch",
            "bb": {
                "machine": "test-machine",                                                                                           
                "distro": "test-distro",
                "deploydir": "tmp/test/deploy",
                "docker": "test-registry/test-image:0.1",
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
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_settings).expect("Failed to parse settings");
        let data: WsBitbakeData = WsBitbakeData::from_str(json_build_config, &settings).expect("Failed to parse product data");
        assert_eq!(data.machine(), "test-machine");
        assert_eq!(data.distro(), "test-distro");
        assert_eq!(data.docker_image(), "test-registry/test-image:0.1");
        assert_eq!(data.build_dir(), PathBuf::from(String::from("/workspace/builds/test-name")));
        assert_eq!(data.build_config_dir(), PathBuf::from(String::from("/workspace/builds/test-name/conf")));
        assert_eq!(data.deploy_dir(), PathBuf::from(String::from("/workspace/builds/test-name/tmp/test/deploy")));
        assert_eq!(data.bblayers_conf_path(), PathBuf::from(String::from("/workspace/builds/test-name/conf/bblayers.conf")));
        assert_eq!(data.local_conf_path(), PathBuf::from(String::from("/workspace/builds/test-name/conf/local.conf")));
        assert!(!data.bblayers_conf().is_empty());
        assert_eq!(&data.bblayers_conf, &vec![
            String::from("BB_LAYERS_CONF_TEST_LINE_1"),
            String::from("BB_LAYERS_CONF_TEST_LINE_2"),
            String::from("BB_LAYERS_CONF_TEST_LINE_3")
        ]);
        assert!(!data.local_conf().is_empty());
        assert_eq!(&data.local_conf, &vec![
            String::from("BB_LOCAL_CONF_TEST_LINE_1"),
            String::from("BB_LOCAL_CONF_TEST_LINE_2"),
            String::from("BB_LOCAL_CONF_TEST_LINE_3")
        ]);
        assert_eq!(data.sstate_dir(), PathBuf::from(String::from("/workspace/.cache/test-arch/sstate-cache")));
        assert_eq!(data.dl_dir(), PathBuf::from(String::from("/workspace/.cache/download")));
        assert_eq!(data.poky_dir(), PathBuf::from(String::from("/workspace/layers/poky")));
        assert_eq!(data.oe_init_file(), PathBuf::from(String::from("/workspace/layers/poky/oe-init-build-env")));
    }

    #[test]
    fn test_ws_context_data_default() {
        let json_default_build_config = r#"
        {                                                                                                                   
            "version": "4"
        }"#;
        let data: WsContextData = WsContextData::from_str(json_default_build_config).expect("Failed to parse context data");
        assert_eq!(data.get_ctx_value("MACHINE"), "NA");
        assert_eq!(data.get_ctx_value("ARCH"), "NA");
        assert_eq!(data.get_ctx_value("DISTRO"), "NA");
        assert_eq!(data.get_ctx_value("VARIANT"), "NA");
        assert_eq!(data.get_ctx_value("PRODUCT_NAME"), "NA");
        assert_eq!(
            data.get_ctx_path("BB_BUILD_DIR"),
            PathBuf::from("")
        );
        assert_eq!(
            data.get_ctx_path("BB_DEPLOY_DIR"),
            PathBuf::from("")
        );
        assert_eq!(
            data.get_ctx_path("ARTIFACTS_DIR"),
            PathBuf::from("")
        );
        assert_eq!(
            data.get_ctx_path("BUILDS_DIR"),
            PathBuf::from("")
        );
        assert_eq!(data.get_ctx_path("WORK_DIR"), PathBuf::from(""));
        assert_eq!(data.get_ctx_value("PLATFORM_VERSION"), "0.0.0");
        assert_eq!(data.get_ctx_value("BUILD_NUMBER"), "0");
        assert_eq!(data.get_ctx_value("PLATFORM_RELEASE"), "0.0.0-0");
        assert_eq!(data.get_ctx_value("BUILD_SHA"), "dev");
        assert_eq!(data.get_ctx_value("RELEASE_BUILD"), "0");
        assert_eq!(data.get_ctx_value("BUILD_VARIANT"), "dev");
        assert_eq!(data.get_ctx_value("ARCHIVER"), "0");
        assert_eq!(data.get_ctx_value("DEBUG_SYMBOLS"), "0");
    }

    #[test]
    fn test_ws_context_data_overwrite() {
        let json_settings: &str = r#"
        {
            "version": "4"
        }"#;
        let json_build_config = r#"
        {                                                                                                                   
            "version": "4",
            "context": [
                "KEY1=value1",
                "KEY2=value2",
                "KEY3=value3" 
            ]
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_settings).expect("Failed to parse settings");
        let mut data: WsContextData = WsContextData::from_str(json_build_config).expect("Failed to parse context data");
        let ctx_built_in_variables: IndexMap<String, String> = indexmap! {
            "MACHINE".to_string() => "test-machine".to_string(),
            "ARCH".to_string() => "test-arch".to_string(),
            "DISTRO".to_string() => "test-distro".to_string(),
            "VARIANT".to_string() => "test-variant".to_string(),
            "PRODUCT_NAME".to_string() => "test".to_string(),
            "WORK_DIR".to_string() => settings.work_dir().to_string_lossy().to_string(),
        };
        data.update(&ctx_built_in_variables);
        assert_eq!(data.get_ctx_value("MACHINE"), "test-machine");
        assert_eq!(data.get_ctx_value("ARCH"), "test-arch");
        assert_eq!(data.get_ctx_value("DISTRO"), "test-distro");
        assert_eq!(data.get_ctx_value("VARIANT"), "test-variant");
        assert_eq!(data.get_ctx_value("PRODUCT_NAME"), "test");
        assert_eq!(data.get_ctx_value("KEY1"), "value1");
        assert_eq!(data.get_ctx_value("KEY2"), "value2");
        assert_eq!(data.get_ctx_value("KEY3"), "value3");
        assert_eq!(data.get_ctx_path("WORK_DIR"), settings.work_dir());

    }

    #[test]
    fn test_ws_build_data_default() {
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        assert_eq!(data.version(), "4");
        assert_eq!(data.name(), "NA");
    }

    #[test]
    fn test_ws_build_data_no_tasks() {
        let json_build_config = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let data: WsBuildData = Helper::setup_build_data(&work_dir, Some(json_build_config), None);
        let json_data: Value =
            JsonFileReader::parse(json_build_config).expect("Failed to parse json");
        let tasks: IndexMap<String, WsTaskHandler> =
            data.get_tasks(&json_data).expect("Failed to parse tasks");
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_ws_build_data_tasks_error() {
        let json_build_config: &str = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "tasks": "error"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let data: WsBuildData = Helper::setup_build_data(&work_dir, Some(json_build_config), None);
        let json_data: Value =
            JsonFileReader::parse(json_build_config).expect("Failed to parse json");
        let result: Result<IndexMap<String, WsTaskHandler>, BError> = data.get_tasks(&json_data);
        match result {
            Ok(_rconfig) => {
                panic!("We should have recived an error because we have no valid config!");
            }
            Err(e) => {
                assert_eq!(
                    e.to_string(),
                    String::from("Invalid 'task' node in build config. No 'tasks' node found in build config")
                );
            }
        }
    }

    #[test]
    fn test_ws_build_data_task() {
        let json_task_str: &str = r#"
        { 
            "index": "2",
            "name": "task-name",
            "type": "bitbake",
            "recipes": [
                "test-image"
            ]
        }"#;
        let json_build_config: &str = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let data: WsBuildData = Helper::setup_build_data(&work_dir, Some(json_build_config), None);
        let json_data: Value = JsonFileReader::parse(json_task_str).expect("Failed to parse json");
        let task: WsTaskHandler = data.get_task(&json_data).expect("Failed to parse task");
        assert_eq!(task.data().build_dir(), &PathBuf::from("/workspace/builds/test-name"));
        assert_eq!(task.data().name(), "task-name");
    }

    #[test]
    fn test_ws_build_data_task_expand_ctx() {
        let json_task_str: &str = r#"
        { 
            "index": "2",
            "name": "task-name",
            "type": "bitbake",
            "recipes": [
                "${RECIPE_NAME}"
            ]
        }"#;
        let json_build_config: &str = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "context": [
                "RECIPE_NAME=test-image"
            ]
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let data: WsBuildData = Helper::setup_build_data(&work_dir, Some(json_build_config), None);
        let json_data: Value = JsonFileReader::parse(json_task_str).expect("Failed to parse json");
        let task: WsTaskHandler = data.get_task(&json_data).expect("Failed to parse task");
        assert_eq!(task.data().recipes(), &vec!["test-image"]);
    }

    #[test]
    fn test_ws_build_tasks() {
        let json_build_config: &str = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "tasks": { 
                "task1": {
                    "index": "1",
                    "name": "task1",
                    "type": "non-bitbake"
                },
                "task2": {
                    "index": "2",
                    "name": "task2",
                    "type": "non-bitbake"
                }
            }
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let data: WsBuildData = Helper::setup_build_data(&work_dir, Some(json_build_config), None);
        let json_data: Value =
            JsonFileReader::parse(json_build_config).expect("Failed to parse json");
        let tasks: IndexMap<String, WsTaskHandler> =
            data.get_tasks(&json_data).expect("Failed to parse tasks");
        assert!(!tasks.is_empty());
        let mut i: usize = 1;
        tasks.iter().for_each(|(name, task)| {
            assert_eq!(name, &format!("task{}", i));
            assert_eq!(task.data().name(), &format!("task{}", i));
            i += 1;
        });
    }

    #[test]
    fn test_ws_build_data_artifacts_error() {
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let json_artifact_config: &str = r#"
        { 
            "index": "2",
            "name": "task2",
            "type": "non-bitbake",
            "artifacts": "error"
        }"#;
        let data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let json_data: Value =
            JsonFileReader::parse(json_artifact_config).expect("Failed to parse json");
        let result: Result<Vec<WsArtifactsHandler>, BError> =
            data.get_artifacts(&json_data, &task_build_dir);
        match result {
            Ok(_rconfig) => {
                panic!("We should have recived an error because we have no valid config!");
            }
            Err(e) => {
                assert_eq!(
                    e.to_string(),
                    String::from("Invalid 'artifact' node in build config. No 'artifacts' array node found in build config")
                );
            }
        }
    }

    #[test]
    fn test_ws_build_data_artifact() {
        let json_artifact_config: &str = r#"
        {
            "type": "manifest",
            "name": "test-manifest",
            "content": {
                "key1": "value1",
                "key2": "value2",
                "data": {
                    "key3": "value3"
                }
            }
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let json_data: Value =
            JsonFileReader::parse(json_artifact_config).expect("Failed to parse json");
        let artifacts: WsArtifactsHandler = data
            .get_artifact(&json_data, &task_build_dir)
            .expect("Failed to parse artifacts");
        assert_eq!(artifacts.data().atype(), &AType::Manifest);
        assert_eq!(artifacts.data().name(), "test-manifest");
    }

    #[test]
    fn test_ws_build_data_expand_artifact() {
        let json_build_config: &str = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "context": [
                "MANIFEST_FILE_NAME=test-manifest"
            ]
        }"#;
        let json_artifact_config: &str = r#"
        {
            "type": "manifest",
            "name": "${MANIFEST_FILE_NAME}",
            "content": {
                "key1": "value1",
                "key2": "value2",
                "data": {
                    "key3": "value3"
                }
            }
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let data: WsBuildData = Helper::setup_build_data(&work_dir, Some(json_build_config), None);
        let json_data: Value =
            JsonFileReader::parse(json_artifact_config).expect("Failed to parse json");
        let artifact: WsArtifactsHandler = data
            .get_artifact(&json_data, &task_build_dir)
            .expect("Failed to parse artifacts");
        assert_eq!(artifact.data().atype(), &AType::Manifest);
        assert_eq!(artifact.data().name(), "test-manifest");
    }

    #[test]
    fn test_ws_build_data_artifacts() {
        let json_artifacts_config: &str = r#"
        {
            "artifacts": [
                {
                    "source": "file1.txt",
                    "dest": "file1.txt"
                },
                {
                    "source": "file2.txt",
                    "dest": "file2.txt"
                }
            ]
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let json_data: Value =
            JsonFileReader::parse(json_artifacts_config).expect("Failed to parse json");
        let artifacts: Vec<WsArtifactsHandler> = data
            .get_artifacts(&json_data, &task_build_dir)
            .expect("Failed to parse artifacts");
        assert!(!artifacts.is_empty());
        let mut i: usize = 1;
        artifacts.iter().for_each(|a| {
            assert_eq!(a.data().atype(), &AType::File);
            assert_eq!(a.data().source(), &PathBuf::from(format!("/workspace/task/dir/file{}.txt", i)));
            i += 1;
        });
    }
}
