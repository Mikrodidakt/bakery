use crate::cli::Cli;
use crate::collector::{Collected, Collector, CollectorFactory};
use crate::error::BError;
use crate::workspace::WsArtifactsHandler;

use std::path::PathBuf;

pub struct DirectoryCollector<'a> {
    artifact: &'a WsArtifactsHandler,
    cli: Option<&'a Cli>,
}

impl<'a> Collector for DirectoryCollector<'a> {
    fn collect(&self, src: &PathBuf, dest: &PathBuf) -> Result<Vec<Collected>, BError> {
        let directory_name: &str = self.artifact.data().name();
        let directory_path: PathBuf = dest.join(PathBuf::from(directory_name));
        let mut collected: Vec<Collected> = vec![];
        self.info(
            self.cli,
            format!("Collecting directory '{}'", directory_name),
        );
        for child in self.artifact.children().iter() {
            let collector: Box<dyn Collector> = CollectorFactory::create(child, None)?;
            let mut c: Vec<Collected> = collector.collect(src, &directory_path)?;
            collected.append(&mut c);
        }
        self.info(
            self.cli,
            format!("All artifacts collected at '{}'", directory_path.display()),
        );
        Ok(collected)
    }

    fn verify_attributes(&self) -> Result<(), BError> {
        if self.artifact.data().name().is_empty() || self.artifact.children().is_empty() {
            return Err(BError::ValueError(String::from(
                "Directory node requires name and list of artifacts!",
            )));
        }
        Ok(())
    }
}

impl<'a> DirectoryCollector<'a> {
    pub fn new(artifact: &'a WsArtifactsHandler, cli: Option<&'a Cli>) -> Self {
        DirectoryCollector { artifact, cli }
    }
}

#[cfg(test)]
mod tests {
    use crate::collector::{Collected, Collector, DirectoryCollector};
    use crate::configs::Context;
    use crate::data::WsBuildData;
    use crate::helper::Helper;
    use crate::workspace::WsArtifactsHandler;

    use indexmap::{indexmap, IndexMap};
    use std::path::PathBuf;
    use tempdir::TempDir;

    #[test]
    fn test_directory_collector_dir() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let files: Vec<PathBuf> = vec![
            task_build_dir.clone().join("file1.txt"),
            task_build_dir.clone().join("file2.txt"),
        ];
        let json_artifacts_config: &str = r#"
        {
            "type": "directory",
            "name": "dirname",
            "artifacts": [
                {
                    "source": "file1.txt"
                },
                {
                    "source": "file2.txt",
                    "dest": "dest/dest-file.txt"
                }
            ]
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let artifacts: WsArtifactsHandler = Helper::setup_collector_test_ws(
            &work_dir,
            &task_build_dir,
            &files,
            &build_data,
            json_artifacts_config,
        );
        let collector: DirectoryCollector = DirectoryCollector::new(&artifacts, None);
        assert!(collector.verify_attributes().is_ok());
        let artifacts_dir: PathBuf = build_data.settings().artifacts_dir();
        let collected: Vec<Collected> = collector
            .collect(&task_build_dir, &artifacts_dir)
            .expect("Failed to collect artifacts");
        assert_eq!(
            &collected,
            &vec![
                Collected {
                    src: task_build_dir.clone().join("file1.txt"),
                    dest: artifacts_dir.clone().join("dirname/file1.txt")
                },
                Collected {
                    src: task_build_dir.clone().join("file2.txt"),
                    dest: artifacts_dir.clone().join("dirname/dest/dest-file.txt")
                },
            ]
        );
        for c in collected.iter() {
            assert!(c.dest.exists());
        }
    }

    #[test]
    fn test_directory_collector_nested() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let files: Vec<PathBuf> = vec![
            task_build_dir.clone().join("file1.txt"),
            task_build_dir.clone().join("file2.txt"),
            task_build_dir.clone().join("file3.txt"),
        ];
        let json_artifacts_config: &str = r#"
        {
            "type": "directory",
            "name": "dirname1",
            "artifacts": [
                {
                    "source": "file1.txt"
                },
                {
                    "source": "file2.txt",
                    "dest": "dest/dest-file.txt"
                },
                {
                    "type": "directory",
                    "name": "dirname2",
                    "artifacts": [
                        {
                            "source": "file3.txt"
                        }
                    ]
                }
            ]
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let artifacts: WsArtifactsHandler = Helper::setup_collector_test_ws(
            &work_dir,
            &task_build_dir,
            &files,
            &build_data,
            json_artifacts_config,
        );
        let collector: DirectoryCollector = DirectoryCollector::new(&artifacts, None);
        assert!(collector.verify_attributes().is_ok());
        let artifacts_dir: PathBuf = build_data.settings().artifacts_dir();
        let collected: Vec<Collected> = collector
            .collect(&task_build_dir, &artifacts_dir)
            .expect("Failed to collect artifacts");
        assert_eq!(
            &collected,
            &vec![
                Collected {
                    src: task_build_dir.clone().join("file1.txt"),
                    dest: artifacts_dir.clone().join("dirname1/file1.txt")
                },
                Collected {
                    src: task_build_dir.clone().join("file2.txt"),
                    dest: artifacts_dir.clone().join("dirname1/dest/dest-file.txt")
                },
                Collected {
                    src: task_build_dir.clone().join("file3.txt"),
                    dest: artifacts_dir.clone().join("dirname1/dirname2/file3.txt")
                },
            ]
        );
        for c in collected.iter() {
            assert!(c.dest.exists());
        }
    }

    #[test]
    fn test_directory_collector_context() {
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
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let mut artifacts: WsArtifactsHandler = Helper::setup_collector_test_ws(
            &work_dir,
            &task_build_dir,
            &files,
            &build_data,
            json_artifacts_config,
        );
        let variables: IndexMap<String, String> = indexmap! {
            "DIR1".to_string() => "dir1".to_string(),
            "DIR2".to_string() => "dir2".to_string(),
            "DEST_FILE".to_string() => "dest-file.txt".to_string(),
            "SRC_FILE".to_string() => "src-file.txt".to_string(),
        };
        let context: Context = Context::new(&variables);
        artifacts.expand_ctx(&context).unwrap();
        let collector: DirectoryCollector = DirectoryCollector::new(&artifacts, None);
        assert!(collector.verify_attributes().is_ok());
        let artifacts_dir: PathBuf = build_data.settings().artifacts_dir();
        let collected: Vec<Collected> = collector
            .collect(&task_build_dir, &artifacts_dir)
            .expect("Failed to collect artifacts");
        assert_eq!(
            &collected,
            &vec![
                Collected {
                    src: task_build_dir.clone().join("file1.txt"),
                    dest: artifacts_dir.clone().join("dir1/file1.txt")
                },
                Collected {
                    src: task_build_dir.clone().join("file2.txt"),
                    dest: artifacts_dir.clone().join("dir1/dest/dest-file.txt")
                },
                Collected {
                    src: task_build_dir.clone().join("src-file.txt"),
                    dest: artifacts_dir.clone().join("dir1/dir2/src-file.txt")
                },
            ]
        );
        for c in collected.iter() {
            assert!(c.dest.exists());
        }
    }
}
