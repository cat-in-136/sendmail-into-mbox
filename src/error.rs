use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to read the mail message from stream")]
    ReadMessage(#[source] io::Error),
    #[error("Spool output error")]
    WriteSpool(#[source] io::Error),
}
