use crate::collector::{
    Collector,
    CollectorFactory,
    Collected,
};
use crate::cli::Cli;
use crate::error::BError;
use crate::fs::Archiver;
use crate::workspace::WsArtifactsHandler;

use std::path::PathBuf;
use tempdir::TempDir;

pub struct ArchiveCollector<'a> {
    artifact: &'a WsArtifactsHandler,
    cli: Option<&'a Cli>,
}

impl<'a> Collector for ArchiveCollector<'a> {
    fn collect(&self, src: &PathBuf, dest: &PathBuf) -> Result<Vec<Collected>, BError> {
        let archive_name: &str = self.artifact.data().name();
        let archive_path: PathBuf = dest.join(PathBuf::from(archive_name));
        let temp_dir: TempDir =
            TempDir::new("bakery-archiver")?;
        let archive_tmp_dir: PathBuf = PathBuf::from(temp_dir.path());
        let mut collected: Vec<Collected> = vec![];

        self.info(self.cli, format!("Collecting archive files for '{}'", archive_name));

        for child in self.artifact.children().iter() {
            let collector: Box<dyn Collector> = CollectorFactory::create(child, None)?;
            let mut c: Vec<Collected> = collector.collect(src, &archive_tmp_dir)?;
            collected.append(&mut c);
        }

        let files: Vec<PathBuf> = collected.iter().map(|f| {
            f.dest.clone()
        }).collect();
        let archiver: Archiver = Archiver::new(&archive_path)?;
        archiver.add_files(&files, &archive_tmp_dir)?;
        self.info(self.cli, format!("All artifacts collected in archive '{}'", archive_path.display()));

        Ok(vec![Collected { src: PathBuf::from(""), dest: archive_path} ])
    }

    fn verify_attributes(&self) -> Result<(), BError> {
        if self.artifact.data().name().is_empty()
            || self.artifact.children().is_empty() {
                return Err(BError::ValueError(String::from("Archive node requires name and list of artifacts!")));
        }
        Ok(())
    }
}

impl<'a> ArchiveCollector<'a> {
    pub fn new(artifact: &'a WsArtifactsHandler, cli: Option<&'a Cli>) -> Self {
        ArchiveCollector {
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
        ArchiveCollector,
        Collector,
        Collected,
    };
    
    use tempdir::TempDir;
    use std::path::PathBuf;

    #[test]
    fn test_archive_collector_files() {
        let archive_name: &str = "archive.zip";
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
            "type": "archive",
            "name": "archive.zip",
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
            json_artifacts_config);
        let collector: ArchiveCollector = ArchiveCollector::new(&artifacts, None);
        assert!(collector.verify_attributes().is_ok());
        let artifacts_dir: PathBuf = build_data.settings().artifacts_dir();
        let collected: Vec<Collected> = collector.collect(&task_build_dir, &artifacts_dir).expect("Failed to collect artifacts");
        assert_eq!(collected, vec![
            Collected { src: PathBuf::from(""), dest: artifacts_dir.clone().join(archive_name) }
        ]);
        for c in collected.iter() {
            assert!(c.dest.exists());
        }
    }

    #[test]
    fn test_archive_collector_nested() {
        let archive_name: &str = "archive.zip";
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
            "type": "archive",
            "name": "archive.zip",
            "artifacts": [
                {
                    "source": "file1.txt"
                },
                {
                    "source": "file2.txt",
                    "dest": "dest/dest-file.txt"
                },
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
        let collector: ArchiveCollector = ArchiveCollector::new(&artifacts, None);
        assert!(collector.verify_attributes().is_ok());
        let artifacts_dir: PathBuf = build_data.settings().artifacts_dir();
        let collected: Vec<Collected> = collector.collect(&task_build_dir, &artifacts_dir).expect("Failed to collect artifacts");
        assert_eq!(collected, vec![
            Collected { src: PathBuf::from(""), dest: artifacts_dir.clone().join(archive_name) }
        ]);
        for c in collected.iter() {
            assert!(c.dest.exists());
        }
    }
}