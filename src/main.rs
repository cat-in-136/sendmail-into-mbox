mod error;
mod mail_message;

use crate::error::Error as AppError;
use crate::mail_message::MailMessage;
use std::process::exit;

fn sendmail_main() -> Result<(), AppError> {
    let mut mbox =
        MailMessage::new_from_stream(std::io::stdin()).map_err(|e| AppError::ReadMessage(e))?;
    mbox.fix_mail_headers();

    mbox.write_to_mbox(std::io::stdout(), "MAILER-DAEMON@localhost")
        .map_err(|e| AppError::WriteSpool(e))?;

    Ok(())
}

fn main() {
    sendmail_main().unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        exit(1);
    });
}
