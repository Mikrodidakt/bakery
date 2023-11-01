use crate::collector::Collector;
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
}

impl<'a> Collector for ManifestCollector<'a> {
    fn collect(&self, _src: &PathBuf, dest: &PathBuf) -> Result<Vec<PathBuf>, BError> {
        let manifest_path: PathBuf = dest.join(PathBuf::from(self.artifact.data().name()));
        let manifest: Manifest = Manifest::new(&manifest_path)?;
        manifest.write(self.artifact.data().manifest())?;

        Ok(vec![manifest_path])
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
    pub fn new(artifact: &'a WsArtifactsHandler) -> Self {
        ManifestCollector {
            artifact,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::workspace::WsArtifactsHandler;
    use crate::data::WsBuildData;
    use crate::helper::Helper;
    use crate::collector::{ManifestCollector, Collector};
    use tempdir::TempDir;
    use std::fs::File;
    use std::io::{self, Read};

    #[test]
    fn test_ws_artifacts_manifest() {
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
        let collector: ManifestCollector = ManifestCollector::new(&artifacts);
        let collected: Vec<PathBuf> = collector.collect(&task_build_dir, &build_data.settings().artifacts_dir()).expect("Failed to collect artifacts");
        let manifest_file: PathBuf = build_data.settings().artifacts_dir().clone().join("manifest.json");
        assert_eq!(collected, vec![manifest_file.clone()]);
        assert!(manifest_file.exists());
        let mut file: File = File::open(&manifest_file).expect("Failed to open manifest file!");
        let mut contents: String = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read manifest file!");
        assert_eq!(artifacts.data().manifest(), contents);
    }
}