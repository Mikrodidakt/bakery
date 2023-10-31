use std::{fmt, path::StripPrefixError};

use thiserror::Error;
use serde_json::Error;
use zip::result::ZipError;

#[derive(Error, Debug)]
pub enum BError {
  #[error("{0}")]
  ParseError(String),
  #[error("Invalid 'artifact' node in build config. {0}")]
  ParseArtifactsError(String),
  #[error("Invalid 'task' node in build config. {0}")]
  ParseTasksError(String),
  #[error("Invalid 'manifest' node in build config. {0}")]
  ParseManifestError(String),
  #[error("Failed to parse JSON. {0}")]
  JsonParseError(String),
  #[error("{0}")]
  IOError(String),
  #[error("{0}")]
  ValueError(String),
  #[error("{0}")]
  WsError(String),
  #[error("{0}")]
  CliError(String),
  #[error("{0}")]
  ArchiverError(String),
  #[error("{0}")]
  CmdError(String),
  #[error("{0}")]
  CtxKeyError(String),
  #[error("{0}")]
  CollectorError(String),
}

impl std::convert::From<serde_json::Error> for BError {
    fn from(err: serde_json::Error) -> Self {
        BError::JsonParseError(err.to_string())
    }
}

impl std::convert::From<std::io::Error> for BError {
    fn from(err: std::io::Error) -> Self {
        BError::IOError(err.to_string())
    }
}

impl std::convert::From<StripPrefixError> for BError {
    fn from(err: StripPrefixError) -> Self {
        BError::ArchiverError(err.to_string())
    }
}

impl std::convert::From<ZipError> for BError {
    fn from(err: ZipError) -> Self {
        BError::ArchiverError(err.to_string())
    }
}