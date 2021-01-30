use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to load config file `{0}`: {1}")]
    ConfigIo(PathBuf, #[source] io::Error),
    #[error("Failed to load config file `{0}`: {1}")]
    ConfigToml(PathBuf, #[source] toml::de::Error),
    #[error("Failed to read the mail message from stream")]
    ReadMessage(#[source] io::Error),
    #[error("Failed to write to `{0}`: {1}")]
    WriteSpool(PathBuf, #[source] io::Error),
    #[error("Failed to write to the standard output: {0}")]
    WriteSpoolStdOut(#[source] io::Error),
}
