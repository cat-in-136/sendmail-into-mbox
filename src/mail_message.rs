use chrono::{Local, Utc};
use std::io::{BufRead, BufReader, BufWriter, Error, Read, Write};

/// Representation of a mail message
#[derive(Debug)]
pub struct MailMessage {
    /// Mail headers.
    ///
    /// ## Note
    /// Each header shall end with `\r\n`.
    headers: Vec<String>,
    /// Mail body.
    body: String,
}

impl MailMessage {
    /// Create a mail message from `stream` in the `sendmail` manner.
    pub fn new_from_stream(stream: impl Read) -> Result<Self, Error> {
        let mut headers = Vec::new();
        let reader = BufReader::new(stream);
        let mut buffer = Vec::new();

        for line in reader.lines() {
            match line {
                Ok(v) if headers.is_empty() && v.is_empty() => {
                    headers.append(&mut buffer);
                    debug_assert!(buffer.is_empty());
                }
                Ok(v) => buffer.push(format!("{}\r\n", v)),
                Err(err) => return Err(err),
            }
        }
        let body = buffer.concat();

        Ok(Self { headers, body })
    }

    /// Fix mail headers.
    ///
    /// * Remove `Bcc:` headers
    /// * Add `Message-ID:` header if absent
    /// * Add `Date:` header if absent
    pub fn fix_mail_headers(&mut self) {
        // Remove Bcc: headers
        self.headers.retain(|v| !v.starts_with("Bcc:"));

        if !self.headers.iter().any(|v| v.starts_with("Message-ID:")) {
            let now = Utc::now();
            let message_id = format!(
                "{}.{}.{}@localhost",
                now.timestamp(),
                now.timestamp_subsec_millis(),
                now.timestamp_subsec_micros()
            );
            self.headers
                .insert(0, format!("Message-ID: <{}>\r\n", message_id));
        }
        if !self.headers.iter().any(|v| v.starts_with("Date:")) {
            self.headers
                .insert(0, format!("Date: {}\r\n", Local::now().to_rfc2822()));
        }
    }

    /// Write the mail message to `stream` in RFC4155 Mbox format.
    ///
    /// # Note
    /// `sender` should not be empty, in which case use `MAILER-DAEMON@localhost` instead.
    pub fn write_to_mbox(&self, stream: impl Write, sender: &str) -> Result<(), Error> {
        let mut writer = BufWriter::new(stream);
        let now = Local::now();

        // Each message in the mbox database MUST be immediately preceded
        // by a single separator line
        writer.write_fmt(format_args!(
            // The exact character sequence of "From";
            // a single Space character (0x20);
            "From {} {}\n",
            // the email address of the message sender (as obtained from the
            // message envelope or other authoritative source), conformant
            // with the "addr-spec" syntax from RFC 2822;
            sender,
            // a timestamp indicating the UTC date and time when the message
            // was originally received, conformant with the syntax of the
            // traditional UNIX 'ctime' output sans timezone
            now.naive_utc().format("%c")
        ))?;

        fn escape_line(line: &str) -> String {
            // database MUST use a single Line-Feed character (0x0A) as the
            // end-of-line sequence, and MUST NOT use a Carriage-Return/Line-
            // Feed pair
            let mut line = line.replace("\r\n", "\n");

            // The program then copies the message, applying >From quoting
            // to each line.  >From quoting ensures that the resulting
            // lines are not From_ lines:  the program prepends a > to any
            // From_ line, >From_ line, >>From_ line, >>>From_ line, etc.
            // http://qmail.org/man/man5/mbox.html
            if line.trim_start_matches(">").starts_with("From ") {
                line.replace_range(0..0, ">")
            }

            line
        }

        // received        =   "Received:" *received-token ";" date-time CRLF
        // https://tools.ietf.org/html/rfc5322#section-3.6.7
        writer.write_fmt(format_args!(
            "Received: by localhost with {}; {}\n",
            env!("CARGO_PKG_NAME"),
            now.to_rfc2822()
        ))?;

        for header in &self.headers {
            writer.write(escape_line(header).as_bytes())?;
        }

        // The body is simply a sequence of characters that
        // follows the header and is separated from the header by an empty line
        writer.write("\n".as_bytes())?;

        for line in self.body.lines() {
            writer.write(escape_line(line).as_bytes())?;
            writer.write("\n".as_bytes())?;
        }

        // Each message in the database MUST be terminated by an empty
        // line, containing a single end-of-line marker.
        writer.write("\n".as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new_from_stream() {
        let input = "From: from\r\nTo: to\r\nSubject: subject\r\n\r\nbody\r\nbody\r\n".as_bytes();
        let mail = MailMessage::new_from_stream(input).unwrap();
        assert_eq!(
            mail.headers,
            vec!["From: from\r\n", "To: to\r\n", "Subject: subject\r\n"]
        );
        assert_eq!(mail.body, "body\r\nbody\r\n");
    }

    #[test]
    fn test_new_from_stream_with_broken_mail_format() {
        let input = "body\nbody".as_bytes();
        let mail = MailMessage::new_from_stream(input).unwrap();
        assert!(mail.headers.is_empty());
        assert_eq!(mail.body, "body\r\nbody\r\n");
    }

    #[test]
    fn test_fix_mail_headers() {
        let mut mail = MailMessage {
            headers: vec!["Bcc: secret\r\n".to_string()],
            body: "\r\n".to_string(),
        };
        mail.fix_mail_headers();
        assert!(mail.headers[0].starts_with("Date: "));
        assert!(mail.headers[1].starts_with("Message-ID: "));
        assert_eq!(mail.headers.len(), 2);
    }

    #[test]
    fn test_write_to_mbox() {
        let mail = MailMessage {
            // http://qmail.org/man/man5/mbox.html
            headers: vec!["From: djb\r\n".to_string(), "To: god\r\n".to_string()],
            body: "From now through August I'll be doing beta testing.\r\n\
            Thanks for your interest.\r\n"
                .to_string(),
        };
        let mut mbox = Vec::new();
        mail.write_to_mbox(&mut mbox, "djb").unwrap();

        let mbox_str = std::str::from_utf8(&mbox).unwrap();
        let mbox_lines = mbox_str.splitn(3, "\n").collect::<Vec<_>>();

        assert!(mbox_lines[0].starts_with("From djb "));
        assert!(mbox_lines[1].starts_with(concat!(
            "Received: by localhost with ",
            env!("CARGO_PKG_NAME"),
            "; "
        )));
        assert_eq!(
            mbox_lines[2],
            "From: djb\n\
             To: god\n\
             \n\
             >From now through August I'll be doing beta testing.\n\
             Thanks for your interest.\n\
             \n"
        );
        assert_eq!(mbox_lines.len(), 3);
    }
}
