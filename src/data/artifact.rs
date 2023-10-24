use serde_json::Value;
use std::path::PathBuf;

use crate::configs::Context;
use crate::error::BError;
use crate::data::WsBuildData;
use crate::configs::Config;

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

#[cfg(test)]
mod tests {
}
