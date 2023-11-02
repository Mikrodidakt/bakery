use crate::collector::{
    Collector,
    Collected,
};
use crate::cli::Cli;
use crate::error::BError;
use crate::data::{
    AType,
    WsArtifactData,
};
use crate::workspace::WsArtifactsHandler;
use crate::fs::Manifest;

use std::path::PathBuf;

pub struct ManifestCollector<'a> {
    artifact: &'a WsArtifactsHandler,
    cli: Option<&'a Cli>,
}

impl<'a> Collector for ManifestCollector<'a> {
    fn collect(&self, _src: &PathBuf, dest: &PathBuf) -> Result<Vec<Collected>, BError> {
        let manifest_file: &str = self.artifact.data().name();
        let manifest_path: PathBuf = dest.join(PathBuf::from(manifest_file));
        let manifest: Manifest = Manifest::new(&manifest_path)?;
        
        self.info(self.cli, format!("Creating manifest file '{}'", manifest_file));
        manifest.write(self.artifact.data().manifest())?;
        self.info(self.cli, format!("Manifest file '{}' available at {}", manifest_file, manifest_path.display()));

        Ok(vec![Collected { src: PathBuf::from(""), dest: manifest_path }])
    }

    fn verify_attributes(&self) -> Result<(), BError> {
        if self.artifact.data().name().is_empty()
            || self.artifact.data().manifest().is_empty() {
                return Err(BError::ValueError(String::from("Manifest node requires name and manifest content!")));
        }
        Ok(())
    }
}

impl<'a> ManifestCollector<'a> {
    pub fn new(artifact: &'a WsArtifactsHandler, cli: Option<&'a Cli>) -> Self {
        ManifestCollector {
            artifact,
            cli,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::configs::Context;
    use crate::workspace::WsArtifactsHandler;
    use crate::data::WsBuildData;
    use crate::helper::Helper;
    use crate::collector::{
        ManifestCollector,
        Collector,
        Collected,
    };
    use tempdir::TempDir;
    use std::fs::File;
    use std::io::{self, Read};
    use std::path::PathBuf;
    use indexmap::{indexmap, IndexMap};

    #[test]
    fn test_manifest_collector_content() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let files: Vec<PathBuf> = vec![];
        let json_artifacts_config: &str = r#"
        {
            "type": "manifest",
            "name": "manifest.json",
            "content": {
                "test1": "value1",
                "test2": "value2",
                "test3": "value3",
                "data": {
                    "test4": "value4",
                    "test5": "value5",
                    "test6": "value6"
                }

            }
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let artifacts: WsArtifactsHandler = Helper::setup_collector_test_ws(
            &work_dir,
            &task_build_dir,
            &files,
            &build_data,
            json_artifacts_config);
        let collector: ManifestCollector = ManifestCollector::new(&artifacts, None);
        let collected: Vec<Collected> = collector.collect(&task_build_dir, &build_data.settings().artifacts_dir()).expect("Failed to collect artifacts");
        let manifest_file: PathBuf = build_data.settings().artifacts_dir().clone().join("manifest.json");
        assert_eq!(collected, vec![Collected { src: PathBuf::from(""), dest: manifest_file.clone() }]);
        assert!(manifest_file.exists());
        let mut file: File = File::open(&manifest_file).expect("Failed to open manifest file!");
        let mut contents: String = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read manifest file!");
        assert_eq!(artifacts.data().manifest(), contents);
    }

    #[test]
    fn test_manifest_collector_context() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let files: Vec<PathBuf> = vec![];
        let json_artifacts_config: &str = r#"
        {
            "type": "manifest",
            "name": "${MANIFEST_FILE}",
            "content": {
                "test1": "${TEST_VALUE1}",
                "test2": "value2",
                "test3": "${TEST_VALUE3}",
                "data": {
                    "test4": "value4",
                    "test5": "value5",
                    "test6": "${TEST_VALUE6}"
                }
            }
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let mut artifacts: WsArtifactsHandler = Helper::setup_collector_test_ws(
            &work_dir,
            &task_build_dir,
            &files,
            &build_data,
            json_artifacts_config);
        let variables: IndexMap<String, String> = indexmap! {
                "MANIFEST_FILE".to_string() => "ctxmanifest.json".to_string(),
                "TEST_VALUE1".to_string() => "var1".to_string(),
                "TEST_VALUE3".to_string() => "var2".to_string(),
                "TEST_VALUE6".to_string() => "var3".to_string(),
        };
        let context: Context = Context::new(&variables);
        artifacts.expand_ctx(&context);
        let collector: ManifestCollector = ManifestCollector::new(&artifacts, None);
        let collected: Vec<Collected> = collector.collect(&task_build_dir, &build_data.settings().artifacts_dir()).expect("Failed to collect artifacts");
        let manifest_file: PathBuf = build_data.settings().artifacts_dir().clone().join("ctxmanifest.json");
        assert_eq!(collected, vec![Collected { src: PathBuf::from(""), dest: manifest_file.clone() }]);
        assert!(manifest_file.exists());
        let mut file: File = File::open(&manifest_file).expect("Failed to open manifest file!");
        let mut contents: String = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read manifest file!");
        let json_manifest_content: &str = r#"{"data":{"test4":"value4","test5":"value5","test6":"var3"},"test1":"var1","test2":"value2","test3":"var2"}"#;
        assert_eq!(json_manifest_content, contents);
    }
}