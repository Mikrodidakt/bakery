use crate::collector::{
    Collector,
    Collected,
};
use crate::cli::Cli;
use crate::error::BError;
use crate::workspace::WsArtifactsHandler;

use std::path::{PathBuf, Path};

pub struct FileCollector<'a> {
    artifact: &'a WsArtifactsHandler,
    cli: Option<&'a Cli>,
}

impl<'a> Collector for FileCollector<'a> {
    fn collect(&self, src: &PathBuf, dest: &PathBuf) -> Result<Vec<Collected>, BError> {
        let dest_str: &str = self.artifact.data().dest();
        let src_path: PathBuf = src.join(PathBuf::from(self.artifact.data().source()));
        let dest_path: PathBuf = dest.join(PathBuf::from(dest_str));
        let files: Vec<PathBuf> = self.list_files(&src_path)?;
        let base_dir: &Path = src_path.parent().unwrap();
        let mut collected: Vec<Collected> = vec![];

        for f in files.iter() {
            let mut dest_file: PathBuf = dest_path.clone();
            if self.is_dir(&dest_path, dest_str) {
                let src_prefix: &Path = f.strip_prefix(base_dir)?;
                //println!("Prefix: {}", src_prefix.display());
                dest_file = dest_file.join(PathBuf::from(src_prefix));
            }

            if !f.exists() {
                return Err(BError::IOError(format!("File {} dose not exists", f.display())));
            }

            self.info(self.cli, format!("Copy file {} => {}", f.display(), dest_file.display()));
            std::fs::create_dir_all(dest_file.parent().unwrap())?;
            std::fs::copy(f, &dest_file)?;
            collected.push(Collected { src: f.clone(), dest: dest_file.clone() });
        }

        Ok(collected)
    }

    fn verify_attributes(&self) -> Result<(), BError> {
        if self.artifact.data().source().is_empty() {
            return Err(BError::ValueError(String::from("File node requires source attribute!")));
        }

        Ok(())
    }
}

impl<'a> FileCollector<'a> {
    fn is_dir(&self, dest_path: &PathBuf, dest_str: &str) -> bool {
        let file_name = dest_path.file_name();

        if dest_str.is_empty() {
            return true;
        }

        if file_name == None {
            return true;
        }

        if dest_path.extension() != None {
            return false;
        }

        return true;
    }

    fn list_files(&self, glob_pattern_path: &PathBuf) -> Result<Vec<PathBuf>, BError> {
        let mut files: Vec<PathBuf> = vec![];
        match glob_pattern_path.to_str() {
            Some(pattern) => {
                //println!("pattern: {:?}", pattern);
                for entry in glob::glob(pattern)? {
                    match entry {
                        Ok(path) => {
                            //println!("{:?}", path.clone().display());
                            if path.is_dir() {
                                let mut f: Vec<PathBuf> = self.list_files(&path.join("*"))?;
                                files.append(&mut f);
                            } else {
                                files.push(path);
                            }
                        },
                        Err(e) => {
                            return Err(BError::CollectorError(e.to_string()));
                        }
                    }
                }
            },
            None => {
                files.push(glob_pattern_path.clone());
            }
        }
        Ok(files)
    }

    pub fn new(artifact: &'a WsArtifactsHandler, cli: Option<&'a Cli>) -> Self {
        FileCollector {
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
    use crate::collector::{FileCollector, Collector, Collected};
    use crate::configs::Context;

    use tempdir::TempDir;
    use std::path::PathBuf;
    use indexmap::{indexmap, IndexMap};

    #[test]
    fn test_file_collector_source() {
        let src_file_name: &str = "file.txt";
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let files: Vec<PathBuf> = vec![
            task_build_dir.clone().join(src_file_name)
        ];
        let json_artifacts_config: &str = r#"
        {
            "source": "file.txt"
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let artifacts: WsArtifactsHandler = Helper::setup_collector_test_ws(
            &work_dir,
            &task_build_dir,
            &files,
            &build_data,
            json_artifacts_config);
        let collector: FileCollector = FileCollector::new(&artifacts, None);
        let collected: Vec<Collected> = collector.collect(&task_build_dir, &build_data.settings().artifacts_dir()).expect("Failed to collect artifacts");
        let dest: PathBuf = build_data.settings().artifacts_dir().clone().join(src_file_name);
        assert_eq!(collected, vec![
            Collected { src: task_build_dir.join(src_file_name), dest: dest.clone() }
        ]);
        assert!(dest.exists());
    }

    #[test]
    fn test_file_collector_file_dest() {
        let src_file_name: &str = "src.txt";
        let dest_file_name: &str = "dest.txt";
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let files: Vec<PathBuf> = vec![
            task_build_dir.clone().join(src_file_name)
        ];
        let json_artifacts_config: &str = r#"
        {
            "source": "src.txt",
            "dest": "dest.txt"
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let artifacts: WsArtifactsHandler = Helper::setup_collector_test_ws(
            &work_dir,
            &task_build_dir,
            &files,
            &build_data,
            json_artifacts_config);
        let collector: FileCollector = FileCollector::new(&artifacts, None);
        let collected: Vec<Collected> = collector.collect(&task_build_dir, &build_data.settings().artifacts_dir()).expect("Failed to collect artifacts");
        let dest: PathBuf = build_data.settings().artifacts_dir().clone().join(dest_file_name);
        assert_eq!(collected, vec![
            Collected { src: task_build_dir.join(src_file_name), dest: dest.clone() }
        ]);
        assert!(dest.exists());
    }

    #[test]
    fn test_file_collector_dest_dir() {
        let src_file_name: &str = "src.txt";
        let dest_file_name: &str = "dest/file.txt";
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let files: Vec<PathBuf> = vec![
            task_build_dir.clone().join(src_file_name)
        ];
        let json_artifacts_config: &str = r#"
        {
            "source": "src.txt",
            "dest": "dest/file.txt"
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let artifacts: WsArtifactsHandler = Helper::setup_collector_test_ws(
            &work_dir,
            &task_build_dir,
            &files,
            &build_data,
            json_artifacts_config);
        let collector: FileCollector = FileCollector::new(&artifacts, None);
        let collected: Vec<Collected> = collector.collect(&task_build_dir, &build_data.settings().artifacts_dir()).expect("Failed to collect artifacts");
        let dest: PathBuf = build_data.settings().artifacts_dir().clone().join(dest_file_name);
        assert_eq!(collected, vec![
            Collected { src: task_build_dir.join(src_file_name), dest: dest.clone() },
        ]);
        assert!(dest.exists());
    }

    #[test]
    fn test_file_collector_src_dir() {
        let src_file_name: &str = "src/src-file.txt";
        let dest_file_name: &str = "dest/dest-file.txt";
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let files: Vec<PathBuf> = vec![
            task_build_dir.clone().join(src_file_name)
        ];
        let json_artifacts_config: &str = r#"
        {
            "source": "src/src-file.txt",
            "dest": "dest/dest-file.txt"
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let artifacts: WsArtifactsHandler = Helper::setup_collector_test_ws(
            &work_dir,
            &task_build_dir,
            &files,
            &build_data,
            json_artifacts_config);
        let collector: FileCollector = FileCollector::new(&artifacts, None);
        let collected: Vec<Collected> = collector.collect(&task_build_dir, &build_data.settings().artifacts_dir()).expect("Failed to collect artifacts");
        let dest: PathBuf = build_data.settings().artifacts_dir().clone().join(dest_file_name);
        assert_eq!(collected, vec![
            Collected { src: task_build_dir.join(src_file_name), dest: dest.clone() },
        ]);
        assert!(dest.exists());
    }

    #[test]
    fn test_file_collector_src_glob() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let files: Vec<PathBuf> = vec![
            task_build_dir.clone().join("src/sub/file1.txt"),
            task_build_dir.clone().join("src/sub/dir1/file2.txt"),
            task_build_dir.clone().join("src/sub/dir2/file3.txt"),
            task_build_dir.clone().join("src/sub/dir3/file4.txt"),
            task_build_dir.clone().join("src/sub/dir4/dir5/file5.txt"),
        ];
        let json_artifacts_config: &str = r#"
        {
            "source": "src/sub/*",
            "dest": "dest/"
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let artifacts: WsArtifactsHandler = Helper::setup_collector_test_ws(
            &work_dir,
            &task_build_dir,
            &files,
            &build_data,
            json_artifacts_config);
        let collector: FileCollector = FileCollector::new(&artifacts, None);
        let artifacts_dir: PathBuf = build_data.settings().artifacts_dir();
        let collected: Vec<Collected> = collector.collect(&task_build_dir, &artifacts_dir).expect("Failed to collect artifacts");
        assert_eq!(&collected, &vec![
            Collected { src: task_build_dir.clone().join("src/sub/dir1/file2.txt"), dest: artifacts_dir.clone().join("dest/dir1/file2.txt") },
            Collected { src: task_build_dir.clone().join("src/sub/dir2/file3.txt"), dest: artifacts_dir.clone().join("dest/dir2/file3.txt") },
            Collected { src: task_build_dir.clone().join("src/sub/dir3/file4.txt"), dest: artifacts_dir.clone().join("dest/dir3/file4.txt") },
            Collected { src: task_build_dir.clone().join("src/sub/dir4/dir5/file5.txt"), dest: artifacts_dir.clone().join("dest/dir4/dir5/file5.txt") },
            Collected { src: task_build_dir.clone().join("src/sub/file1.txt"), dest: artifacts_dir.clone().join("dest/file1.txt") },
        ]);
        for c in collected.iter() {
            assert!(c.dest.exists());
        }
    }

    #[test]
    fn test_file_collector_context() {
        let src_file_name: &str = "src/dir1/dir2/src-file.txt";
        let dest_file_name: &str = "dest/dir3/dest-file.txt";
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let files: Vec<PathBuf> = vec![
            task_build_dir.clone().join(src_file_name)
        ];
        let json_artifacts_config: &str = r#"
        {
            "source": "src/$#[DIR1]/$#[DIR2]/$#[SRC_FILE]",
            "dest": "dest/$#[DIR3]/$#[DEST_FILE]"
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let mut artifacts: WsArtifactsHandler = Helper::setup_collector_test_ws(
            &work_dir,
            &task_build_dir,
            &files,
            &build_data,
            json_artifacts_config);
        let variables: IndexMap<String, String> = indexmap! {
                "DIR1".to_string() => "dir1".to_string(),
                "DIR2".to_string() => "dir2".to_string(),
                "DIR3".to_string() => "dir3".to_string(),
                "DEST_FILE".to_string() => "dest-file.txt".to_string(),
                "SRC_FILE".to_string() => "src-file.txt".to_string(),
        };
        let context: Context = Context::new(&variables);
        artifacts.expand_ctx(&context);
        let collector: FileCollector = FileCollector::new(&artifacts, None);
        let collected: Vec<Collected> = collector.collect(&task_build_dir, &build_data.settings().artifacts_dir()).expect("Failed to collect artifacts");
        let dest: PathBuf = build_data.settings().artifacts_dir().clone().join(dest_file_name);
        assert_eq!(collected, vec![
            Collected { src: task_build_dir.join(src_file_name), dest: dest.clone() },
        ]);
        assert!(dest.exists());
    }
}