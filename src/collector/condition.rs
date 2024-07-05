use crate::collector::{
    Collector,
    CollectorFactory,
    Collected,
};
use crate::cli::Cli;
use crate::error::BError;
use crate::workspace::WsArtifactsHandler;

use std::path::PathBuf;

pub struct ConditionCollector<'a> {
    artifact: &'a WsArtifactsHandler,
    cli: Option<&'a Cli>,
}

impl<'a> Collector for ConditionCollector<'a> {
    fn collect(&self, src: &PathBuf, dest: &PathBuf) -> Result<Vec<Collected>, BError> {
        let condition: bool = self.artifact.data().condition();
        let mut collected: Vec<Collected> = vec![];
        if condition {
            for child in self.artifact.children().iter() {
                let collector: Box<dyn Collector> = CollectorFactory::create(child, None)?;
                let mut c: Vec<Collected> = collector.collect(src, dest)?;
                collected.append(&mut c);
            }
        } else {
            self.info(self.cli, "Skipping collecting condition false".to_string());
        }
        Ok(collected)
    }

    fn verify_attributes(&self) -> Result<(), BError> {
        if self.artifact.data().name().is_empty()
            || self.artifact.children().is_empty() {
                return Err(BError::ValueError(String::from("Directory node requires name and list of artifacts!")));
        }
        Ok(())
    }
}

impl<'a> ConditionCollector<'a> {
    pub fn new(artifact: &'a WsArtifactsHandler, cli: Option<&'a Cli>) -> Self {
        ConditionCollector {
            artifact,
            cli,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::workspace::WsArtifactsHandler;
    use crate::data::WsBuildData;
    use crate::helper::Helper;
    use crate::collector::{
        ConditionCollector,
        Collector,
        Collected
    };
    use crate::configs::Context;

    use tempdir::TempDir;
    use std::path::PathBuf;
    use indexmap::{indexmap, IndexMap};

    #[test]
    fn test_conditional_collector_true() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let files: Vec<PathBuf> = vec![
            task_build_dir.clone().join("file1.txt"),
        ];
        let json_artifacts_config: &str = r#"
        {
            "type": "condition",
            "condition": "true",
            "artifacts": [
                {
                    "source": "file1.txt"
                }
            ]
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let artifacts: WsArtifactsHandler = Helper::setup_collector_test_ws(
            &work_dir,
            &task_build_dir,
            &files,
            &build_data,
            json_artifacts_config);
        let collector: ConditionCollector = ConditionCollector::new(&artifacts, None);
        let artifacts_dir: PathBuf = build_data.settings().artifacts_dir();
        let collected: Vec<Collected> = collector.collect(&task_build_dir, &artifacts_dir).expect("Failed to collect artifacts");
        assert_eq!(&collected, &vec![
            Collected { src: task_build_dir.clone().join("file1.txt"), dest: artifacts_dir.clone().join("file1.txt") },
        ]);
        for c in collected.iter() {
            assert!(c.dest.exists());
        }
    }

    #[test]
    fn test_conditional_collector_false() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let files: Vec<PathBuf> = vec![
            task_build_dir.clone().join("file1.txt"),
        ];
        let json_artifacts_config: &str = r#"
        {
            "type": "condition",
            "condition": "false",
            "artifacts": [
                {
                    "source": "file1.txt"
                }
            ]
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let artifacts: WsArtifactsHandler = Helper::setup_collector_test_ws(
            &work_dir,
            &task_build_dir,
            &files,
            &build_data,
            json_artifacts_config);
        let collector: ConditionCollector = ConditionCollector::new(&artifacts, None);
        let artifacts_dir: PathBuf = build_data.settings().artifacts_dir();
        let collected: Vec<Collected> = collector.collect(&task_build_dir, &artifacts_dir).expect("Failed to collect artifacts");
        assert!(collected.is_empty());
    }

    #[test]
    fn test_conditional_collector_ctx() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let files: Vec<PathBuf> = vec![
            task_build_dir.clone().join("file1.txt"),
            task_build_dir.clone().join("file2.txt"),
            task_build_dir.clone().join("src-file.txt"),
        ];
        let json_artifacts_config: &str = r#"
        {
            "type": "condition",
            "condition": "$#[CONDITION]",
            "artifacts": [
                {
                    "type": "directory",
                    "name": "$#[DIR1]",
                    "artifacts": [
                        {
                            "source": "file1.txt"
                        },
                        {
                            "source": "file2.txt",
                            "dest": "dest/$#[DEST_FILE]"
                        },
                        {
                            "type": "directory",
                            "name": "$#[DIR2]",
                            "artifacts": [
                                {
                                    "source": "$#[SRC_FILE]"
                                }
                            ]
                        }
                    ]
                }
            ]   
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let mut artifacts: WsArtifactsHandler = Helper::setup_collector_test_ws(
            &work_dir,
            &task_build_dir,
            &files,
            &build_data,
            json_artifacts_config);
        let variables: IndexMap<String, String> = indexmap! {
            "CONDITION".to_string() => "true".to_string(),
            "DIR1".to_string() => "dir1".to_string(),
            "DIR2".to_string() => "dir2".to_string(),
            "DEST_FILE".to_string() => "dest-file.txt".to_string(),
            "SRC_FILE".to_string() => "src-file.txt".to_string(),
        };
        let context: Context = Context::new(&variables);
        artifacts.expand_ctx(&context).unwrap();
        let collector: ConditionCollector = ConditionCollector::new(&artifacts, None);
        let artifacts_dir: PathBuf = build_data.settings().artifacts_dir();
        let collected: Vec<Collected> = collector.collect(&task_build_dir, &artifacts_dir).expect("Failed to collect artifacts");
        assert_eq!(&collected, &vec![
            Collected { src: task_build_dir.clone().join("file1.txt"), dest: artifacts_dir.clone().join("dir1/file1.txt") },
            Collected { src: task_build_dir.clone().join("file2.txt"), dest: artifacts_dir.clone().join("dir1/dest/dest-file.txt") },
            Collected { src: task_build_dir.clone().join("src-file.txt"), dest: artifacts_dir.clone().join("dir1/dir2/src-file.txt") },
        ]);
        for c in collected.iter() {
            assert!(c.dest.exists());
        }
    }
}