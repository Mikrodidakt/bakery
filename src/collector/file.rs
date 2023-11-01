use crate::collector::Collector;
use crate::cli::Cli;
use crate::error::BError;
use crate::data::{
    AType,
    WsArtifactData,
};
use crate::workspace::WsArtifactsHandler;

use std::path::{PathBuf, Path};

pub struct FileCollector<'a> {
    artifact: &'a WsArtifactsHandler,
}

impl<'a> Collector for FileCollector<'a> {
    fn collect(&self, src: &PathBuf, dest: &PathBuf) -> Result<Vec<PathBuf>, BError> {
        let src_path: PathBuf = src.join(PathBuf::from(self.artifact.data().source()));
        let dest_path: PathBuf = dest.join(PathBuf::from(self.artifact.data().dest()));
        let files: Vec<PathBuf> = self.list_files(&src_path)?;
        let base_dir: &Path = src_path.parent().unwrap();
        
        for f in files.iter() {
            let mut dest_file: PathBuf = dest_path.clone();
            if !dest_file.file_name().unwrap_or_default().is_empty() {
                let src_prefix: &Path = f.strip_prefix(base_dir)?;
                //println!("Prefix: {}", src_prefix.display());
                dest_file = dest_file.join(PathBuf::from(src_prefix));
            }

            if !f.exists() {
                return Err(BError::IOError(format!("File {} dose not exists", f.to_string_lossy().to_string())));
            }

            println!("Copy file {} => {}", f.to_string_lossy().to_string(), dest_file.to_string_lossy().to_string());
            std::fs::create_dir_all(dest_file.parent().unwrap())?;
            std::fs::copy(f, &dest_file)?;
        }

        Ok(files)
    }
}

impl<'a> FileCollector<'a> {
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

    pub fn new(artifact: &'a WsArtifactsHandler) -> Self {
        FileCollector {
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
    use crate::collector::{FileCollector, Collector};
    use tempdir::TempDir;

    #[test]
    fn test_ws_artifacts_file_source() {
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
        let collector: FileCollector = FileCollector::new(&artifacts);
        let collected: Vec<PathBuf> = collector.collect(&task_build_dir, &build_data.settings().artifacts_dir()).expect("Failed to collect artifacts");
        assert_eq!(collected, vec![task_build_dir.join(src_file_name)]);
        assert!(build_data.settings().artifacts_dir().clone().join(src_file_name).exists());
    }

    #[test]
    fn test_ws_artifacts_file_dest() {
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
        let collector: FileCollector = FileCollector::new(&artifacts);
        let collected: Vec<PathBuf> = collector.collect(&task_build_dir, &build_data.settings().artifacts_dir()).expect("Failed to collect artifacts");
        assert_eq!(collected, vec![task_build_dir.join(src_file_name)]);
        assert!(build_data.settings().artifacts_dir().clone().join(dest_file_name).exists());
    }

    #[test]
    fn test_ws_artifacts_dest_dir() {
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
        let collector: FileCollector = FileCollector::new(&artifacts);
        let collected: Vec<PathBuf> = collector.collect(&task_build_dir, &build_data.settings().artifacts_dir()).expect("Failed to collect artifacts");
        assert_eq!(collected, vec![
            task_build_dir.join(src_file_name)
        ]);
        assert!(build_data.settings().artifacts_dir().clone().join(dest_file_name).exists());
    }

    #[test]
    fn test_ws_artifacts_src_dir() {
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
        let collector: FileCollector = FileCollector::new(&artifacts);
        let collected: Vec<PathBuf> = collector.collect(&task_build_dir, &build_data.settings().artifacts_dir()).expect("Failed to collect artifacts");
        assert_eq!(collected, vec![
            task_build_dir.join(src_file_name)
        ]);
        assert!(build_data.settings().artifacts_dir().clone().join(dest_file_name).exists());
    }

    #[test]
    fn test_ws_artifacts_src_glob() {
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
        let collector: FileCollector = FileCollector::new(&artifacts);
        let artifacts_dir: PathBuf = build_data.settings().artifacts_dir();
        let collected: Vec<PathBuf> = collector.collect(&task_build_dir, &artifacts_dir).expect("Failed to collect artifacts");
        assert_eq!(&collected, &vec![
            task_build_dir.clone().join("src/sub/dir1/file2.txt"),
            task_build_dir.clone().join("src/sub/dir2/file3.txt"),
            task_build_dir.clone().join("src/sub/dir3/file4.txt"),
            task_build_dir.clone().join("src/sub/dir4/dir5/file5.txt"),
            task_build_dir.clone().join("src/sub/file1.txt"),
        ]);
        assert!(artifacts_dir.clone().join("dest/file1.txt").exists());
        assert!(artifacts_dir.clone().join("dest/dir1/file2.txt").exists());
        assert!(artifacts_dir.clone().join("dest/dir2/file3.txt").exists());
        assert!(artifacts_dir.clone().join("dest/dir3/file4.txt").exists());
        assert!(artifacts_dir.clone().join("dest/dir4/dir5/file5.txt").exists());
    }
}