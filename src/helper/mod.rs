use crate::workspace::{WsBuildConfigHandler, WsSettingsHandler, Workspace};
use crate::error::BError;
use crate::configs::{WsSettings, BuildConfig, TaskConfig};
use crate::fs::Archiver;

use std::path::{PathBuf, Path};
use std::fs::File;
use std::collections::HashSet;

pub struct Helper;

impl Helper {
    pub fn helper_list_files(archive: &Archiver, work_dir: &Path) -> Result<Vec<PathBuf>, BError> {
        // TODO: the prefered solution would be to use the entries() of the tar::Archive struct
        // but for some reason it will always only return one entry so we are not able to list
        // the files without unpack the content.

        let mut archived_files: Vec<PathBuf> = vec![PathBuf::from("")];
        let unpack_dir: PathBuf = work_dir.join(PathBuf::from("unpack/"));

        if !archive.path().exists() {
            return Err(BError {
                code: 1, // You may set the appropriate error code
                message: format!("No such archive '{}'", archive.path().display()),
            });
        }

        if archive.extension() == "tar" {
            let file: File = File::open(archive.path()).map_err(|err| BError {
                code: 1, // You may set the appropriate error code
                message: format!("Failed to open archive '{}'", err),
            })?;
            let mut tar: tar::Archive<Box<dyn std::io::Read>>;
            if archive.compression() == "gz" {
                tar = tar::Archive::new(Box::new(flate2::read::GzDecoder::new(file)));
            } else if archive.compression() == "bz2" {
                tar = tar::Archive::new(Box::new(bzip2::read::BzDecoder::new(file)));
            } else {
                return Err(BError {
                    code: 0,
                    message: format!("Unsupported compression '{}'!", archive.compression()),
                });
            }

            tar.unpack(unpack_dir.to_str().unwrap()).unwrap();

            archived_files = std::fs::read_dir(&unpack_dir)
                .map_err(|err| BError {
                    code: 1, // You may set the appropriate error code
                    message: format!("Failed unpack archive '{}'", err),
                })?
                .map(|f| {
                    let p = f.unwrap().path();
                    println!("work_dir: {}, unpack_dir: {}, file: {}", work_dir.display(), unpack_dir.display(), p.display());
                    p.strip_prefix(unpack_dir.as_os_str())
                        .expect("Failed to strip prefix")
                        .to_path_buf()
                })
                .collect();
        } else if archive.extension() == "zip" {
        }

        Ok(archived_files)
    }

    pub fn helper_verify_archived_files(expected_files: &Vec<PathBuf>, archived_files: &Vec<PathBuf>, work_dir: &Path) {
            // strip the workdir from the files
            let files: Vec<PathBuf> = expected_files.iter()
                .map(|f| {
                    let p = f
                        .strip_prefix(work_dir.as_os_str())
                        .expect("Failed to strip prefix")
                        .to_path_buf();
                    p
                }).collect();
            
            // Convert to HashSet so we are not failing if the order of the vectors are not matching
            let expected: HashSet<_> = files.iter().map(|p| p.as_path()).collect();
            let archived: HashSet<_> = archived_files.iter().map(|p| p.as_path()).collect();
            assert_eq!(expected, archived);
    }

    pub fn setup_task_config(json_test_str: &str) -> TaskConfig {
        let result: Result<TaskConfig, BError> = TaskConfig::from_str(json_test_str);
        match result {
            Ok(rconfig) => {
                rconfig
            }
            Err(e) => {
                eprintln!("Error parsing tasks from build config: {}", e);
                panic!();
            } 
        }
    }

    pub fn setup_ws_settings(json_test_str: &str) -> WsSettings {
        let result: Result<WsSettings, BError> = WsSettings::from_str(json_test_str);
        let settings: WsSettings;
        match result {
            Ok(rsettings) => {
                settings = rsettings;
            }
            Err(e) => {
                eprintln!("Error parsing JSON: {}", e);
                panic!();
            } 
        }
        settings
    }

    pub fn setup_build_config(json_test_str: &str) -> BuildConfig {
        let result: Result<BuildConfig, BError> = BuildConfig::from_str(json_test_str);
        match result {
            Ok(rconfig) => {
                rconfig
            }
            Err(e) => {
                eprintln!("Error parsing build config: {}", e);
                panic!();
            } 
        }
    }

    pub fn setup_ws_config_handler(test_work_dir: &str, json_settings: &str, json_build_config: &str) -> WsBuildConfigHandler {
        let work_dir: PathBuf = PathBuf::from(test_work_dir);
        let settings: WsSettingsHandler = WsSettingsHandler::new(
            work_dir,
            Helper::setup_ws_settings(json_settings),
        );
        let result: Result<WsBuildConfigHandler, BError> = WsBuildConfigHandler::from_str(&settings, json_build_config);
        match result {
            Ok(ws_config) => {
                ws_config
            }
            Err(e) => {
                eprintln!("Error parsing build config: {}", e);
                panic!();
            } 
        }
    }

    pub fn setup_ws(test_work_dir: &str, json_settings: &str, json_build_config: &str) -> Workspace {
        let work_dir: PathBuf = PathBuf::from(test_work_dir);
        let ws_config: WsSettings = Self::setup_ws_settings(json_settings);
        let build_config: BuildConfig = Self::setup_build_config(json_build_config);
        Workspace::new(Some(work_dir), ws_config, build_config)
    }
}