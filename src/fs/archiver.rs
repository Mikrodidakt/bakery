use std::fs::File;
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
    
        let suffixes: Vec<&str> = archive_name.split('.').collect();
        let name: String = path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .ok_or_else(|| BError::ArchiverError("Archive file name is not valid UTF-8!".to_string()))?
            .to_string();
    
        let mut archive_type = String::new();
        let mut compression = String::new();
    
        if suffixes.len() < 2 {
            return Err(BError::ArchiverError("Archive must have an extension!".to_string()));
        }
    
        for i in (0..suffixes.len()).rev() {
            match suffixes[i] {
                "zip" => {
                    archive_type = "zip".to_string();
                    break;
                }
                "tar" if i + 1 < suffixes.len() => {
                    archive_type = "tar".to_string();
                    match suffixes[i + 1] {
                        "gz" => {
                            compression = "gz".to_string();
                            break;
                        }
                        "bz2" => {
                            compression = "bz2".to_string();
                            break;
                        }
                        _ => {
                            return Err(BError::ArchiverError(format!("Unsupported compression '{}'!", suffixes[i + 1])));
                        }
                    }
                }
                _ => {}
            }
        }
    
        if archive_type.is_empty() {
            return Err(BError::ArchiverError(format!("Unsupported archive '{}'!", name)));
        }
    
        if archive_type == "tar" && compression.is_empty() {
            return Err(BError::ArchiverError("Archive must have a compression!".to_string()));
        }

        Ok(Archiver {
            path: path.clone(),
            name,
            extension: archive_type,
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
            std::fs::create_dir_all(parent_dir)?;
        }

        // If the archive file exists then we should append the files to the existing
        // archive else we should create the archive file.
        if self.path.exists() {
            // For now let's disable the append until we know for sure that we want
            // to use append file to archive.
            mode = Mode::Write;
            // mode == Mode::Append;
        }

        let archive_file: File = File::create(&self.path)?;

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
                return Err(BError::ArchiverError(format!("Unsupported compression '{}'!", self.compression)));
            }

            tar = tar::Builder::new(enc);
            for path in files {
                let striped_path: PathBuf = path
                    .strip_prefix(work_dir.as_os_str())?
                    .to_path_buf();
                let mut file: File = File::open(path)?;
                tar.append_file(striped_path, &mut file)?;
            }

            tar.finish()?;
        } else if self.extension() == "zip" {
            if mode == Mode::Append {}

            let mut zip: ZipWriter<File> = zip::write::ZipWriter::new(archive_file);

            let mut options: FileOptions = zip::write::FileOptions::default().unix_permissions(0o755);
            options = options.large_file(true);

            for path in files {
                //println!("{}", path.display());
                //println!("{}", work_dir.display());
                let striped_path: PathBuf = path
                    .strip_prefix(work_dir.as_os_str())?
                    .to_path_buf();

                let mut file: File = File::open(path)?;

                zip.start_file(striped_path.to_string_lossy().to_owned(), options)?;

                std::io::copy(&mut file, &mut zip)?;
            }

            zip.finish()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
    fn test_archiver_version_zip() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let archiver_path: PathBuf = path.join("test-x.y.z-archiver.zip");
        let archiver: Archiver = Archiver::new(&archiver_path).expect("Failed to setup archiver!");
        assert_eq!(archiver.name(), "test-x.y.z-archiver.zip");
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

    #[test]
    fn test_archiver_version_tar_bz() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let archiver_path: PathBuf = path.join("test-x.y.z-archiver.tar.bz2");
        let archiver: Archiver = Archiver::new(&archiver_path).expect("Failed to setup archiver!");
        assert_eq!(archiver.name(), "test-x.y.z-archiver.tar.bz2");
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
        assert_eq!(error.to_string(), "Unsupported archive 'test-archiver.gzip'!".to_string());
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
            error.to_string(),
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
            error.to_string(),
            "Unsupported archive 'test-archiver.tar'!".to_string()
        );
    }

    #[test]
    fn test_archiver_file_tar_gz() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: &Path = temp_dir.path();
        let archiver_path: PathBuf = work_dir.join("test-archiver.tar.gz");
        let files: Vec<PathBuf> = vec![
            PathBuf::from(work_dir.join("dir1/file1.txt")),
            PathBuf::from(work_dir.join("file2.txt")),
            PathBuf::from(work_dir.join("dir2/file3.txt")),
            PathBuf::from(work_dir.join("dir3/file4.txt")),
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
            PathBuf::from(work_dir.join("dir2/file1.txt")),
            PathBuf::from(work_dir.join("file2.txt")),
            PathBuf::from(work_dir.join("dir3/file3.txt")),
            PathBuf::from(work_dir.join("dir1/file4.txt")),
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
            PathBuf::from(work_dir.join("dir2/file1.txt")),
            PathBuf::from(work_dir.join("file2.txt")),
            PathBuf::from(work_dir.join("dir3/file3.txt")),
            PathBuf::from(work_dir.join("dir1/file4.txt")),
        ];

        Helper::create_test_files(&files);

        let archiver: Archiver = Archiver::new(&archiver_path).expect("Failed to setup archiver!");
        archiver
            .add_files(&files, work_dir)
            .expect("Failed too create archive test-archive.zip");

        let archived_files: Vec<PathBuf> =
            Helper::list_files_in_archive(&archiver, &work_dir.join(PathBuf::from("unpack/")))
                .expect("Failed to list files in archive");

        // Verify that the archive has been created correctly for the zip archive
        // we are iterating over the content and is collecting the file names
        Helper::verify_archived_files(&files, &archived_files, work_dir);
    }
}
