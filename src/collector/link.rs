use crate::cli::Cli;
use crate::collector::{Collected, Collector};
use crate::error::BError;
use crate::workspace::WsArtifactsHandler;

use std::os::unix::fs;
use std::path::PathBuf;

pub struct LinkCollector<'a> {
    artifact: &'a WsArtifactsHandler,
    cli: Option<&'a Cli>,
}

impl<'a> Collector for LinkCollector<'a> {
    fn collect(&self, src: &PathBuf, dest: &PathBuf) -> Result<Vec<Collected>, BError> {
        let link_name: &str = self.artifact.data().name();
        let src_path: PathBuf = src.join(PathBuf::from(self.artifact.data().source()));
        let link_path: PathBuf = dest.join(PathBuf::from(link_name));
        let mut collected: Vec<Collected> = vec![];

        if !src_path.exists() {
            return Err(BError::IOError(format!(
                "File '{}' dose not exists",
                src_path.display()
            )));
        }

        self.info(
            self.cli,
            format!(
                "Link file {} => {}",
                link_path.display(),
                src_path.display()
            ),
        );
        std::fs::create_dir_all(link_path.parent().unwrap())?;
        if link_path.exists() {
            std::fs::remove_file(link_path.clone())?;
        }
        fs::symlink(&src_path, &link_path)?;
        collected.push(Collected {
            src: src_path.clone(),
            dest: link_path.clone(),
        });

        Ok(collected)
    }

    fn verify_attributes(&self) -> Result<(), BError> {
        if self.artifact.data().source().is_empty() {
            return Err(BError::ValueError(String::from(
                "Link node requires source attribute!",
            )));
        }

        if self.artifact.data().name().is_empty() {
            return Err(BError::ValueError(String::from(
                "Link node requires name attribute!",
            )));
        }

        Ok(())
    }
}

impl<'a> LinkCollector<'a> {
    pub fn new(artifact: &'a WsArtifactsHandler, cli: Option<&'a Cli>) -> Self {
        LinkCollector { artifact, cli }
    }
}

#[cfg(test)]
mod tests {
    use crate::collector::{Collected, Collector, LinkCollector};
    use crate::configs::Context;
    use crate::data::WsBuildData;
    use crate::helper::Helper;
    use crate::workspace::WsArtifactsHandler;

    use indexmap::{indexmap, IndexMap};
    use std::path::PathBuf;
    use tempdir::TempDir;

    #[test]
    fn test_link_collector_source() {
        let src_file_name: &str = "file.txt";
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let files: Vec<PathBuf> = vec![task_build_dir.clone().join(src_file_name)];
        let json_artifacts_config: &str = r#"
        {
            "type": "link",
            "name": "link.txt",
            "source": "file.txt"
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let artifacts: WsArtifactsHandler = Helper::setup_collector_test_ws(
            &work_dir,
            &task_build_dir,
            &files,
            &build_data,
            json_artifacts_config,
        );
        let collector: LinkCollector = LinkCollector::new(&artifacts, None);
        assert!(collector.verify_attributes().is_ok());
        let collected: Vec<Collected> = collector
            .collect(&task_build_dir, &build_data.settings().artifacts_dir())
            .expect("Failed to collect artifacts");
        let dest: PathBuf = build_data
            .settings()
            .artifacts_dir()
            .clone()
            .join("link.txt");
        assert_eq!(
            collected,
            vec![Collected {
                src: task_build_dir.join(src_file_name),
                dest: dest.clone()
            }]
        );
        assert!(dest.exists());
    }

    #[test]
    fn test_link_collector_context() {
        let src_file_name: &str = "src/dir1/dir2/src-file.txt";
        let link_file_name: &str = "link-ctx1-ctx2.txt";
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let files: Vec<PathBuf> = vec![task_build_dir.clone().join(src_file_name)];
        let json_artifacts_config: &str = r#"
        {
            "type": "link",
            "name": "$#[LINK_FILE]",
            "source": "src/$#[DIR1]/$#[DIR2]/$#[SRC_FILE]"
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
                "DIR3".to_string() => "dir3".to_string(),
                "CTX1".to_string() => "ctx1".to_string(),
                "CTX2".to_string() => "ctx2".to_string(),
                "LINK_FILE".to_string() => "link-$#[CTX1]-$#[CTX2].txt".to_string(),
                "SRC_FILE".to_string() => "src-file.txt".to_string(),
        };
        let context: Context = Context::new(&variables);
        artifacts.expand_ctx(&context).unwrap();
        let collector: LinkCollector = LinkCollector::new(&artifacts, None);
        assert!(collector.verify_attributes().is_ok());
        let collected: Vec<Collected> = collector
            .collect(&task_build_dir, &build_data.settings().artifacts_dir())
            .expect("Failed to collect artifacts");
        let dest: PathBuf = build_data
            .settings()
            .artifacts_dir()
            .clone()
            .join(link_file_name);
        assert_eq!(
            collected,
            vec![Collected {
                src: task_build_dir.join(src_file_name),
                dest: dest.clone()
            },]
        );
        assert!(dest.exists());
    }

    #[test]
    fn test_link_collector_no_source() {
        let src_file_name: &str = "file.txt";
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        /*
         * No source file will be created so we can make sure that
         * we catches that and aborts
         */
        let files: Vec<PathBuf> = vec![];
        let json_artifacts_config: &str = r#"
        {
            "type": "link",
            "name": "link.txt",
            "source": "file.txt"
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let artifacts: WsArtifactsHandler = Helper::setup_collector_test_ws(
            &work_dir,
            &task_build_dir,
            &files,
            &build_data,
            json_artifacts_config,
        );
        let collector: LinkCollector = LinkCollector::new(&artifacts, None);
        assert!(collector.verify_attributes().is_ok());
        assert!(!task_build_dir.join(src_file_name).exists());
        let result: Result<Vec<Collected>, crate::error::BError> =
            collector.collect(&task_build_dir, &build_data.settings().artifacts_dir());
        match result {
            Ok(_status) => {
                panic!("We should have recived an error because the source is missing!");
            }
            Err(e) => {
                assert_eq!(
                    e.to_string(),
                    format!(
                        "File '{}' dose not exists",
                        task_build_dir.join(src_file_name).display()
                    )
                );
            }
        }
    }
}
