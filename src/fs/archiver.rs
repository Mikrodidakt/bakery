use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::{write::FileOptions, ZipWriter};

use crate::error::BError;

#[derive(Debug)]
pub struct Archiver {
    path: PathBuf,
    name: String,
    extension: String,
    compression: String,
}

#[derive(Clone, PartialEq, Debug)]
enum Mode {
    Append,
    Write,
}

impl Archiver {
    pub fn new(path: &PathBuf) -> Result<Self, BError> {
        let archive_name = path.file_name().unwrap_or_default().to_string_lossy();
        // We read out all suffixes by splitting the archive name based on '.'
        let suffixes: Vec<String> = archive_name
            .split('.')
            .skip(1) // Skip the first part (the actual file name)
            .map(|suffix| suffix.to_string())
            .collect();
        let name: String = path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .ok_or(BError {
                code: 0,
                message: "Archive file name is not valid UTF-8!".to_string(),
            })?
            .to_string();
        let mut extensions: Vec<String> = suffixes.clone();
        let mut extension: String = suffixes.get(0).unwrap_or(&String::from("")).clone();
        let mut compression: String = "".to_string();

        if let Some(first_suffix) = suffixes.get(0) {
            if ["tar", "zip"].contains(&first_suffix.as_str()) {
                if suffixes.len() > 2 {
                    // This is to make sure that we can handle archive names with . in the name
                    // but also to make sure that we can handle tar.gz or any other tar archive
                    // with compression.
                    if suffixes.last() == Some(&String::from(".zip")) {
                        // If the last is zip we will automatically assume all other suffixes are
                        // just part of the archive name for example archive.name.zip
                        extensions = vec!["zip".to_string()];
                    } else if suffixes.get(suffixes.len() - 2).cloned() == Some(String::from("tar"))
                    {
                        // We take the length of the suffixes array minuz 2 because we assume there will be at
                        // most two suffixes when the archive is tar. We read out the archive suffixe and the
                        // compression suffixe and store them in the suffixe vector.
                        extensions = suffixes.iter().rev().take(2).cloned().collect();
                    } else if suffixes.get(0) == Some(&String::from("tar")) {
                        return Err(BError {
                            code: 0,
                            message: "Archive must have an compression!".to_string(),
                        });
                    }
                }
            } else {
                return Err(BError {
                    code: 0,
                    message: format!("Unsupported archive '{}'!", suffixes.get(0).unwrap()),
                });
            }
        } else {
            return Err(BError {
                code: 0,
                message: "Archive must have an extension!".to_string(),
            });
        }

        extension = extensions.get(0).unwrap().clone();
        if extensions.first() == Some(&String::from("tar")) {
            if extensions.len() < 2 {
                return Err(BError {
                    code: 0,
                    message: "Archive must have an compression!".to_string(),
                });
            }
            compression = extensions.last().unwrap().clone();
            if !["gz", "bz2"].contains(&compression.as_str()) {
                return Err(BError {
                    code: 0,
                    message: format!("Unsupported compression '{}'!", compression),
                });
            }
        }

        Ok(Archiver {
            path: path.clone(),
            name,
            extension,
            compression,
        })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn extension(&self) -> &str {
        &self.extension
    }

    pub fn compression(&self) -> &str {
        &self.compression
    }

    pub fn add_files(&self, files: &Vec<PathBuf>, work_dir: &Path) -> Result<(), BError> {
        let mut mode: Mode = Mode::Write;

        if let Some(parent_dir) = self.path.parent() {
            std::fs::create_dir_all(parent_dir).map_err(|err| BError {
                code: 1, // You may set the appropriate error code
                message: format!("Failed to create paranets '{}'", err),
            })?;
        }

        // If the archive file exists then we should append the files to the existing
        // archive else we should create the archive file.
        if self.path.exists() {
            // For now let's disable the append until we know for sure that we want
            // to use append file to archive.
            mode = Mode::Write;
            // mode == Mode::Append;
        }

        let mut archive_file: File = File::create(&self.path).map_err(|err| BError {
            code: 1, // You may set the appropriate error code
            message: format!("Failed to create archive file '{}'", err),
        })?;

        if self.extension() == "tar" {
            if mode == Mode::Append {}

            let enc: Box<dyn std::io::Write>;
            let mut tar: tar::Builder<Box<dyn std::io::Write>>;
            if self.compression() == "gz" {
                enc = Box::new(flate2::write::GzEncoder::new(
                    archive_file,
                    flate2::Compression::default(),
                ));
            } else if self.compression() == "bz2" {
                enc = Box::new(bzip2::write::BzEncoder::new(
                    archive_file,
                    bzip2::Compression::default(),
                ));
            } else {
                return Err(BError {
                    code: 0,
                    message: format!("Unsupported compression '{}'!", self.compression),
                });
            }

            tar = tar::Builder::new(enc);
            for path in files {
                let striped_path: PathBuf = path
                    .strip_prefix(work_dir.as_os_str())
                    .map_err(|err| BError {
                        code: 1, // You may set the appropriate error code
                        message: format!("Failed to strip prefix: '{}'", err),
                    })?
                    .to_path_buf();
                let mut file: File = File::open(path).map_err(|err| BError {
                    code: 1, // You may set the appropriate error code
                    message: format!("Failed to open file to add to archive: '{}'", err),
                })?;
                tar.append_file(striped_path, &mut file)
                    .map_err(|err| BError {
                        code: 1, // You may set the appropriate error code
                        message: format!("Failed to add file to archive '{}'", err),
                    })?;
            }

            tar.finish().map_err(|err| BError {
                code: 1, // You may set the appropriate error code
                message: format!("Failed to create archive '{}'", err),
            })?;
        } else if self.extension() == "zip" {
            if mode == Mode::Append {}

            let mut zip: ZipWriter<File> = zip::write::ZipWriter::new(archive_file);

            let options: FileOptions = zip::write::FileOptions::default().unix_permissions(0o755);

            for path in files {
                let striped_path: PathBuf = path
                    .strip_prefix(work_dir.as_os_str())
                    .map_err(|err| BError {
                        code: 1, // You may set the appropriate error code
                        message: format!("Failed to strip prefix: '{}'", err),
                    })?
                    .to_path_buf();

                let mut file: File = File::open(path).map_err(|err| BError {
                    code: 1, // You may set the appropriate error code
                    message: format!("Failed to open file to add to archive: '{}'", err),
                })?;

                zip.start_file(striped_path.to_string_lossy().to_owned(), options)
                    .map_err(|err| BError {
                        code: 1, // You may set the appropriate error code
                        message: format!("Failed to initialize file to archive: '{}'", err),
                    })?;

                std::io::copy(&mut file, &mut zip).map_err(|err| BError {
                    code: 1, // You may set the appropriate error code
                    message: format!("Failed to copy file into archive: '{}'", err),
                })?;
            }

            zip.finish().map_err(|err| BError {
                code: 1, // You may set the appropriate error code
                message: format!("Failed to create archive '{}'", err),
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::path::{Path, PathBuf};
    use tempdir::TempDir;

    use crate::error::BError;
    use crate::fs::Archiver;
    use crate::helper::Helper;

    #[test]
    fn test_archiver_zip() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let archiver_path: PathBuf = path.join("test-archiver.zip");
        let archiver: Archiver = Archiver::new(&archiver_path).expect("Failed to setup archiver!");
        assert_eq!(archiver.name(), "test-archiver.zip");
        assert_eq!(archiver.extension(), "zip");
        assert_eq!(archiver.compression(), "");
    }

    #[test]
    fn test_archiver_tar_gz() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let archiver_path: PathBuf = path.join("test-archiver.tar.gz");
        let archiver: Archiver = Archiver::new(&archiver_path).expect("Failed to setup archiver!");
        assert_eq!(archiver.name(), "test-archiver.tar.gz");
        assert_eq!(archiver.extension(), "tar");
        assert_eq!(archiver.compression(), "gz");
    }

    #[test]
    fn test_archiver_tar_bz() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let archiver_path: PathBuf = path.join("test-archiver.tar.bz2");
        let archiver: Archiver = Archiver::new(&archiver_path).expect("Failed to setup archiver!");
        assert_eq!(archiver.name(), "test-archiver.tar.bz2");
        assert_eq!(archiver.extension(), "tar");
        assert_eq!(archiver.compression(), "bz2");
    }

    /*
    #[test]
    fn test_archiver_tar_xz() {
        let temp_dir: TempDir = TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let archiver_path: PathBuf = path.join("test-archiver.tar.xz");
        let archiver: Archiver = Archiver::new(&archiver_path).expect("Failed to setup archiver!");
        assert_eq!(archiver.name(), "test-archiver.tar.xz");
        assert_eq!(archiver.extension(), "tar");
        assert_eq!(archiver.compression(), "xz");
    }
    */

    #[test]
    fn test_archiver_error_unsupported_archive() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let archiver_path: PathBuf = path.join("test-archiver.gzip");
        let error: BError = Archiver::new(&archiver_path)
            .expect_err("We are expecting an error but got an Archiver");
        assert_eq!(error.message, "Unsupported archive 'gzip'!".to_string());
    }

    #[test]
    fn test_archiver_error_unsupported_compression() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let archiver_path: PathBuf = path.join("test-archiver.tar.invalid");
        let error: BError = Archiver::new(&archiver_path)
            .expect_err("We are expecting an error but got an Archiver");
        assert_eq!(
            error.message,
            "Unsupported compression 'invalid'!".to_string()
        );
    }

    #[test]
    fn test_archiver_error_no_compression() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let archiver_path: PathBuf = path.join("test-archiver.tar");
        let error: BError = Archiver::new(&archiver_path)
            .expect_err("We are expecting an error but got an Archiver");
        assert_eq!(
            error.message,
            "Archive must have an compression!".to_string()
        );
    }

    #[test]
    fn test_archiver_file_tar_gz() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: &Path = temp_dir.path();
        let archiver_path: PathBuf = work_dir.join("test-archiver.tar.gz");
        let files: Vec<PathBuf> = vec![
            PathBuf::from(work_dir.clone().join("dir1/file1.txt")),
            PathBuf::from(work_dir.clone().join("file2.txt")),
            PathBuf::from(work_dir.clone().join("dir2/file3.txt")),
            PathBuf::from(work_dir.clone().join("dir3/file4.txt")),
        ];

        Helper::create_test_files(&files);

        let archiver: Archiver = Archiver::new(&archiver_path).expect("Failed to setup archiver!");
        archiver
            .add_files(&files, work_dir)
            .expect("Failed too create archive test-archive.tar.gz");

        // Verify that the archive has been created correctly by unpacking and iterate over
        // all files from the archive and compare with the expected files defined in the files
        // vector. We tried using the entries() for the tar::archive but for some reasone we
        // could not get it to work. We will revisit it at some point later on.
        assert!(archiver_path.exists());

        let archived_files: Vec<PathBuf> =
            Helper::list_files_in_archive(&archiver, &work_dir.join(PathBuf::from("unpack/")))
                .expect("Failed to list files in archive");
        Helper::verify_archived_files(&files, &archived_files, work_dir);
    }

    #[test]
    fn test_archiver_file_tar_bz2() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: &Path = temp_dir.path();
        let archiver_path: PathBuf = work_dir.join("test-archiver.tar.bz2");
        let files: Vec<PathBuf> = vec![
            PathBuf::from(work_dir.clone().join("dir2/file1.txt")),
            PathBuf::from(work_dir.clone().join("file2.txt")),
            PathBuf::from(work_dir.clone().join("dir3/file3.txt")),
            PathBuf::from(work_dir.clone().join("dir1/file4.txt")),
        ];

        Helper::create_test_files(&files);

        let archiver: Archiver = Archiver::new(&archiver_path).expect("Failed to setup archiver!");
        archiver
            .add_files(&files, work_dir)
            .expect("Failed too create archive test-archive.tar.bz2");

        // Verify that the archive has been created correctly by unpacking and iterate over
        // all files from the archive and compare with the expected files defined in the files
        // vector. We tried using the entries() for the tar::archive but for some reasone we
        // could not get it to work. We will revisit it at some point later on.
        assert!(archiver_path.exists());

        let archived_files: Vec<PathBuf> =
            Helper::list_files_in_archive(&archiver, &work_dir.join(PathBuf::from("unpack/")))
                .expect("Failed to list files in archive");
        Helper::verify_archived_files(&files, &archived_files, work_dir);
    }

    #[test]
    fn test_archiver_file_zip() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: &Path = temp_dir.path();
        let archiver_path: PathBuf = work_dir.join("test-archiver.zip");
        let files: Vec<PathBuf> = vec![
            PathBuf::from(work_dir.clone().join("dir2/file1.txt")),
            PathBuf::from(work_dir.clone().join("file2.txt")),
            PathBuf::from(work_dir.clone().join("dir3/file3.txt")),
            PathBuf::from(work_dir.clone().join("dir1/file4.txt")),
        ];

        Helper::create_test_files(&files);

        let archiver: Archiver = Archiver::new(&archiver_path).expect("Failed to setup archiver!");
        archiver
            .add_files(&files, work_dir)
            .expect("Failed too create archive test-archive.zip");

        let archived_files: Vec<PathBuf> =
            Helper::list_files_in_archive(&archiver, &work_dir.join(PathBuf::from("unpack/")))
                .expect("Failed to list files in archive");

        Helper::verify_archived_files(&files, &archived_files, work_dir);
    }
}
