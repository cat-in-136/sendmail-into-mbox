mod mail_message;

use crate::mail_message::MailMessage;

fn main() {
    let mut mbox =
        MailMessage::new_from_stream(std::io::stdin()).expect("Failed to read from stream");
    mbox.fix_mail_headers();

    // TODO spool
    mbox.write_to_mbox(std::io::stdout(), "MAILER-DAEMON@localhost")
        .expect("failed to write to mbox")
}
