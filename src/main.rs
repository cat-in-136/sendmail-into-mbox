mod config;
mod error;
mod mail_message;

use crate::config::Config;
use crate::error::Error as AppError;
use crate::mail_message::MailMessage;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::path::PathBuf;
use std::process::exit;

fn sendmail_main() -> Result<(), AppError> {
    let config = Config::new_from_default_toml_file()?;

    let mut mbox =
        MailMessage::new_from_stream(std::io::stdin()).map_err(|e| AppError::ReadMessage(e))?;
    mbox.fix_mail_headers();

    if config.spool_file == "-" {
        mbox.write_to_mbox(std::io::stdout(), &config.sender)
            .map_err(|e| AppError::WriteSpoolStdOut(e))?;
    } else {
        let file = OpenOptions::new()
            .append(true)
            .create_new(false)
            .open(&config.spool_file)
            .map_err(|e| AppError::WriteSpool(PathBuf::from(&config.spool_file), e))?;
        let mut writer = BufWriter::new(file);
        mbox.write_to_mbox(&mut writer, &config.sender)
            .map_err(|e| AppError::WriteSpool(PathBuf::from(&config.spool_file), e))?;
    }

    Ok(())
}

fn main() {
    sendmail_main().unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        exit(1);
    });
}
