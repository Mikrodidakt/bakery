use std::path::PathBuf;
use std::io::Read;
use std::io::Write;

use crate::data::WsBitbakeData;
use crate::error::BError;
use crate::cli::Cli;

pub struct BitbakeConf {
    build_conf_dir: PathBuf,
    local_conf_path: PathBuf,
    bblayers_conf_path: PathBuf,
    local_conf_content: String,
    bblayers_conf_content: String,
    force: bool,
    bb_variables: Vec<String>
}

impl BitbakeConf {
    fn write_bb_build_conf(&self, path: &PathBuf, content: &str) -> Result<(), BError> {
        let mut file: std::fs::File = std::fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    fn cmp_files(&self, path1: &PathBuf, path2: &PathBuf) -> Result<bool, BError> {
        let mut file1: std::fs::File = std::fs::File::open(path1)?;
        let mut file2: std::fs::File = std::fs::File::open(path2)?;
        let mut content1: String = String::new();
        let mut content2: String = String::new();
        file1.read_to_string(&mut content1)?;
        file2.read_to_string(&mut content2)?;
        
        if content1 != content2 {
            return Ok(false);
        }

        Ok(true)
    }

    fn create_bb_conf_file(&self, cli: &Cli, conf_path: &PathBuf, content: &str, bb_variables: Option<&Vec<String>>, force: bool) -> Result<(), BError> {
        let mut conf_str: String = String::from("# AUTO GENERATED\n");
        conf_str.push_str(content);
        let file_name: String = conf_path.file_name().unwrap_or_default().to_string_lossy().to_string();

        match bb_variables {
            Some(variables) => {
                for line in variables {
                    conf_str.push_str(format!("{}\n", line).as_str());
                }
            },
            None => {}
        }

        if force {
            let msg: String = format!("Autogenerate {}", file_name);
            println!("{}", msg);
            cli.info(msg.clone());
            self.write_bb_build_conf(&conf_path, &conf_str)?;
        } else {
            if conf_path.exists() {
                let tmp_conf_path: PathBuf = conf_path.parent().unwrap().join("tmp.conf");
                self.write_bb_build_conf(&tmp_conf_path, &conf_str)?;
                let identical: bool = self.cmp_files(&tmp_conf_path, conf_path)?;
                if identical {
                    cli.info(format!("{} exists skipping", file_name));
                } else {
                    cli.info(format!("Autogenerate {}", file_name));
                    self.write_bb_build_conf(conf_path, &conf_str)?;    
                }
            } else {
                cli.info(format!("Autogenerate {}", file_name));
                self.write_bb_build_conf(conf_path, &conf_str)?;
            }
        }
        Ok(())  
    }

    pub fn create_bblayers_conf(&self, cli: &Cli) -> Result<(), BError> {
        self.create_bb_conf_file(cli,
            &self.bblayers_conf_path,
            &self.bblayers_conf_content,
            None,
            self.force)
    }

    pub fn create_local_conf(&self, cli: &Cli) -> Result<(), BError> {
        self.create_bb_conf_file(cli,
            &self.local_conf_path,
            &self.local_conf_content,
            Some(&self.bb_variables),
            self.force)
    }

    pub fn create_bitbake_configs(&self, cli: &Cli) -> Result<(), BError> {
        std::fs::create_dir_all(&self.build_conf_dir)?;
        self.create_local_conf(cli)?;
        self.create_bblayers_conf(cli)
    }

    pub fn new(bitbake: &WsBitbakeData, bb_variables: &Vec<String>, force: bool) -> Self {
        Self::construct(
            &bitbake.build_config_dir(),
            &bitbake.local_conf_path(),
            &bitbake.bblayers_conf_path(),
            bitbake.local_conf(),
            bitbake.bblayers_conf(),
            force,
            bb_variables.clone(),
        )
    }

    pub fn construct(
            build_conf_dir: &PathBuf,
            local_conf_path: &PathBuf,
            bblayers_conf_path: &PathBuf,
            local_conf_content: String,
            bblayers_conf_content: String,
            force: bool,
            bb_variables: Vec<String>) -> Self {
        BitbakeConf {
            build_conf_dir: build_conf_dir.clone(),
            local_conf_path: local_conf_path.clone(),
            bblayers_conf_path: bblayers_conf_path.clone(),
            local_conf_content,
            bblayers_conf_content,
            force,
            bb_variables,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use tempdir::TempDir;
    use std::fs::File;
    use std::io::Read;

    use crate::cli::{
        BSystem,
        MockLogger,
        Cli,
    };
    use crate::fs::BitbakeConf;

    #[test]
    fn test_bitbake_conf_create_local_conf() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let bitbake_conf_path: PathBuf = path.join("conf");
        let local_conf_path: PathBuf = bitbake_conf_path.join("local.conf");
        let bblayers_conf_path: PathBuf = bitbake_conf_path.join("bblayers.conf");
        let mut local_conf_content: String = String::new();
        local_conf_content.push_str("LOCAL_CONF_TEST_LINE ?= 1\n");
        local_conf_content.push_str("LOCAL_CONF_TEST_LINE ?= 2\n");
        local_conf_content.push_str("LOCAL_CONF_TEST_LINE ?= 3\n");
        let bblayers_conf_content: String = String::new();
        let bb_variables: Vec<String> = vec![];
        let force: bool = false;
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Autogenerate local.conf".to_string()))
            .once()
            .returning(|_x| ());
        let cli: Cli = Cli::new(
            Box::new(mocked_logger),
            Box::new(BSystem::new()),
            clap::Command::new("bakery"),
            None,
        );
        let conf: BitbakeConf = BitbakeConf::construct(
            &bitbake_conf_path,
            &local_conf_path,
            &bblayers_conf_path,
            local_conf_content.clone(),
            bblayers_conf_content,
            force,
            bb_variables);
        std::fs::create_dir_all(&bitbake_conf_path).expect("Failed to create bitbake conf dir");
        conf.create_local_conf(&cli).expect("Failed to create local.conf");
        let mut file: File = File::open(&local_conf_path).expect("Failed to open local.conf file!");
        let mut contents: String = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read local.conf file!");
        let mut validate_local_conf: String = String::from("# AUTO GENERATED\n");
        validate_local_conf.push_str(&local_conf_content);
        assert_eq!(validate_local_conf, contents);
    }

    #[test]
    fn test_bitbake_conf_create_bblayers_conf() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let bitbake_conf_path: PathBuf = path.join("conf");
        let local_conf_path: PathBuf = bitbake_conf_path.join("local.conf");
        let bblayers_conf_path: PathBuf = bitbake_conf_path.join("bblayers.conf");
        let local_conf_content: String = String::new();
        let mut bblayers_conf_content: String = String::new();
        bblayers_conf_content.push_str("BBLAYERS_CONF_TEST_LINE ?= 1\n");
        bblayers_conf_content.push_str("BBLAYERS_CONF_TEST_LINE ?= 2\n");
        bblayers_conf_content.push_str("BBLAYERS_CONF_TEST_LINE ?= 3\n");
        let bb_variables: Vec<String> = vec![];
        let force: bool = false;
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Autogenerate bblayers.conf".to_string()))
            .once()
            .returning(|_x| ());
        let cli: Cli = Cli::new(
            Box::new(mocked_logger),
            Box::new(BSystem::new()),
            clap::Command::new("bakery"),
            None,
        );
        let conf: BitbakeConf = BitbakeConf::construct(
            &bitbake_conf_path,
            &local_conf_path,
            &bblayers_conf_path,
            local_conf_content,
            bblayers_conf_content.clone(),
            force,
            bb_variables);
        std::fs::create_dir_all(&bitbake_conf_path).expect("Failed to create bitbake conf dir");
        conf.create_bblayers_conf(&cli).expect("Failed to create bblayers.conf");
        let mut file: File = File::open(&bblayers_conf_path).expect("Failed to open bblayers.conf file!");
        let mut contents: String = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read bblayers.conf file!");
        let mut validate_bblayers_conf: String = String::from("# AUTO GENERATED\n");
        validate_bblayers_conf.push_str(&bblayers_conf_content);
        assert_eq!(validate_bblayers_conf, contents);
    }

    #[test]
    fn test_bitbake_conf_bblayers_conf_exists() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let bitbake_conf_path: PathBuf = path.join("conf");
        let local_conf_path: PathBuf = bitbake_conf_path.join("local.conf");
        let bblayers_conf_path: PathBuf = bitbake_conf_path.join("bblayers.conf");
        let local_conf_content: String = String::new();
        let mut bblayers_conf_content: String = String::new();
        bblayers_conf_content.push_str("BBLAYERS_CONF_TEST_LINE ?= 1\n");
        bblayers_conf_content.push_str("BBLAYERS_CONF_TEST_LINE ?= 2\n");
        bblayers_conf_content.push_str("BBLAYERS_CONF_TEST_LINE ?= 3\n");
        let bb_variables: Vec<String> = vec![];
        let force: bool = false;
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Autogenerate bblayers.conf".to_string()))
            .once()
            .returning(|_x| ());
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("bblayers.conf exists skipping".to_string()))
            .once()
            .returning(|_x| ());
        let cli: Cli = Cli::new(
            Box::new(mocked_logger),
            Box::new(BSystem::new()),
            clap::Command::new("bakery"),
            None,
        );
        let conf: BitbakeConf = BitbakeConf::construct(
            &bitbake_conf_path,
            &local_conf_path,
            &bblayers_conf_path,
            local_conf_content,
            bblayers_conf_content.clone(),
            force,
            bb_variables);
        std::fs::create_dir_all(&bitbake_conf_path).expect("Failed to create bitbake conf dir");
        conf.create_bblayers_conf(&cli).expect("Failed to create bblayers.conf");
        conf.create_bblayers_conf(&cli).expect("Failed to create bblayers.conf");        
        let mut file: File = File::open(&bblayers_conf_path).expect("Failed to open bblayers.conf file!");
        let mut contents: String = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read bblayers.conf file!");
        let mut validate_bblayers_conf: String = String::from("# AUTO GENERATED\n");
        validate_bblayers_conf.push_str(&bblayers_conf_content);
        assert_eq!(validate_bblayers_conf, contents);
    }

    #[test]
    fn test_bitbake_conf_local_conf_exists() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let bitbake_conf_path: PathBuf = path.join("conf");
        let local_conf_path: PathBuf = bitbake_conf_path.join("local.conf");
        let bblayers_conf_path: PathBuf = bitbake_conf_path.join("bblayers.conf");
        let mut local_conf_content: String = String::new();
        local_conf_content.push_str("LOCAL_CONF_TEST_LINE ?= 1\n");
        local_conf_content.push_str("LOCAL_CONF_TEST_LINE ?= 2\n");
        local_conf_content.push_str("LOCAL_CONF_TEST_LINE ?= 3\n");
        let bblayers_conf_content: String = String::new();
        let bb_variables: Vec<String> = vec![];
        let force: bool = false;
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Autogenerate local.conf".to_string()))
            .once()
            .returning(|_x| ());
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("local.conf exists skipping".to_string()))
            .once()
            .returning(|_x| ());
        let cli: Cli = Cli::new(
            Box::new(mocked_logger),
            Box::new(BSystem::new()),
            clap::Command::new("bakery"),
            None,
        );
        let conf: BitbakeConf = BitbakeConf::construct(
            &bitbake_conf_path,
            &local_conf_path,
            &bblayers_conf_path,
            local_conf_content.clone(),
            bblayers_conf_content,
            force,
            bb_variables);
        std::fs::create_dir_all(&bitbake_conf_path).expect("Failed to create bitbake conf dir");
        conf.create_local_conf(&cli).expect("Failed to create local.conf");
        conf.create_local_conf(&cli).expect("Failed to create local.conf");
        let mut file: File = File::open(&local_conf_path).expect("Failed to open local.conf file!");
        let mut contents: String = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read local.conf file!");
        let mut validate_local_conf: String = String::from("# AUTO GENERATED\n");
        validate_local_conf.push_str(&local_conf_content);
        assert_eq!(validate_local_conf, contents);
    }

    #[test]
    fn test_bitbake_conf_bblayers_conf_exists_force() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let bitbake_conf_path: PathBuf = path.join("conf");
        let local_conf_path: PathBuf = bitbake_conf_path.join("local.conf");
        let bblayers_conf_path: PathBuf = bitbake_conf_path.join("bblayers.conf");
        let local_conf_content: String = String::new();
        let mut bblayers_conf_content: String = String::new();
        bblayers_conf_content.push_str("BBLAYERS_CONF_TEST_LINE ?= 1\n");
        bblayers_conf_content.push_str("BBLAYERS_CONF_TEST_LINE ?= 2\n");
        bblayers_conf_content.push_str("BBLAYERS_CONF_TEST_LINE ?= 3\n");
        let bb_variables: Vec<String> = vec![];
        let force: bool = true;
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Autogenerate bblayers.conf".to_string()))
            .once()
            .returning(|_x| ());
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Autogenerate bblayers.conf".to_string()))
            .once()
            .returning(|_x| ());
        let cli: Cli = Cli::new(
            Box::new(mocked_logger),
            Box::new(BSystem::new()),
            clap::Command::new("bakery"),
            None,
        );
        let conf: BitbakeConf = BitbakeConf::construct(
            &bitbake_conf_path,
            &local_conf_path,
            &bblayers_conf_path,
            local_conf_content,
            bblayers_conf_content.clone(),
            force,
            bb_variables);
        std::fs::create_dir_all(&bitbake_conf_path).expect("Failed to create bitbake conf dir");
        conf.create_bblayers_conf(&cli).expect("Failed to create bblayers.conf");
        conf.create_bblayers_conf(&cli).expect("Failed to create bblayers.conf");        
        let mut file: File = File::open(&bblayers_conf_path).expect("Failed to open bblayers.conf file!");
        let mut contents: String = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read bblayers.conf file!");
        let mut validate_bblayers_conf: String = String::from("# AUTO GENERATED\n");
        validate_bblayers_conf.push_str(&bblayers_conf_content);
        assert_eq!(validate_bblayers_conf, contents);
    }

    #[test]
    fn test_bitbake_conf_local_conf_exists_force() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let bitbake_conf_path: PathBuf = path.join("conf");
        let local_conf_path: PathBuf = bitbake_conf_path.join("local.conf");
        let bblayers_conf_path: PathBuf = bitbake_conf_path.join("bblayers.conf");
        let mut local_conf_content: String = String::new();
        local_conf_content.push_str("LOCAL_CONF_TEST_LINE ?= 1\n");
        local_conf_content.push_str("LOCAL_CONF_TEST_LINE ?= 2\n");
        local_conf_content.push_str("LOCAL_CONF_TEST_LINE ?= 3\n");
        let mut bblayers_conf_content: String = String::new();
        let bb_variables: Vec<String> = vec![];
        let force: bool = true;
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Autogenerate local.conf".to_string()))
            .once()
            .returning(|_x| ());
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Autogenerate local.conf".to_string()))
            .once()
            .returning(|_x| ());
        let cli: Cli = Cli::new(
            Box::new(mocked_logger),
            Box::new(BSystem::new()),
            clap::Command::new("bakery"),
            None,
        );
        let conf: BitbakeConf = BitbakeConf::construct(
            &bitbake_conf_path,
            &local_conf_path,
            &bblayers_conf_path,
            local_conf_content.clone(),
            bblayers_conf_content,
            force,
            bb_variables);
        std::fs::create_dir_all(&bitbake_conf_path).expect("Failed to create bitbake conf dir");
        conf.create_local_conf(&cli).expect("Failed to create local.conf");
        conf.create_local_conf(&cli).expect("Failed to create local.conf");
        let mut file: File = File::open(&local_conf_path).expect("Failed to open local.conf file!");
        let mut contents: String = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read local.conf file!");
        let mut validate_local_conf: String = String::from("# AUTO GENERATED\n");
        validate_local_conf.push_str(&local_conf_content);
        assert_eq!(validate_local_conf, contents);
    }

    #[test]
    fn test_bitbake_create_confs() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let bitbake_conf_path: PathBuf = path.join("conf");
        let local_conf_path: PathBuf = bitbake_conf_path.join("local.conf");
        let bblayers_conf_path: PathBuf = bitbake_conf_path.join("bblayers.conf");
        let mut local_conf_content: String = String::new();
        local_conf_content.push_str("LOCAL_CONF_TEST_LINE ?= 1\n");
        local_conf_content.push_str("LOCAL_CONF_TEST_LINE ?= 2\n");
        local_conf_content.push_str("LOCAL_CONF_TEST_LINE ?= 3\n");
        let mut bblayers_conf_content: String = String::new();
        bblayers_conf_content.push_str("BBLAYERS_CONF_TEST_LINE ?= 1\n");
        bblayers_conf_content.push_str("BBLAYERS_CONF_TEST_LINE ?= 2\n");
        bblayers_conf_content.push_str("BBLAYERS_CONF_TEST_LINE ?= 3\n");
        let bb_variables: Vec<String> = vec![];
        let force: bool = true;
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Autogenerate local.conf".to_string()))
            .once()
            .returning(|_x| ());
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Autogenerate bblayers.conf".to_string()))
            .once()
            .returning(|_x| ());
        let cli: Cli = Cli::new(
            Box::new(mocked_logger),
            Box::new(BSystem::new()),
            clap::Command::new("bakery"),
            None,
        );
        let conf: BitbakeConf = BitbakeConf::construct(
            &bitbake_conf_path,
            &local_conf_path,
            &bblayers_conf_path,
            local_conf_content.clone(),
            bblayers_conf_content.clone(),
            force,
            bb_variables);
        conf.create_bitbake_configs(&cli).expect("Failed to create conf files");
        let mut file: File = File::open(&local_conf_path).expect("Failed to open local.conf file!");
        let mut contents: String = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read local.conf file!");
        let mut validate_local_conf: String = String::from("# AUTO GENERATED\n");
        validate_local_conf.push_str(&local_conf_content);
        assert_eq!(validate_local_conf, contents);
        let mut file: File = File::open(&bblayers_conf_path).expect("Failed to open bblayers.conf file!");
        let mut contents: String = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read bblayers.conf file!");
        let mut validate_bblayers_conf: String = String::from("# AUTO GENERATED\n");
        validate_bblayers_conf.push_str(&bblayers_conf_content);
        assert_eq!(validate_bblayers_conf, contents);
    }
}