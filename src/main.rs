mod mail_message;

use crate::mail_message::MailMessage;
use getopts::Options;
use std::process::exit;

#[derive(Debug, Default)]
struct CmdOption {
    sender: Option<String>,
    recipients: Vec<String>,
    read_recipients_from_body: bool,
}

impl CmdOption {
    fn new_from_cmd_line(args: Vec<String>) -> Self {
        let mut opts = Options::new();
        opts.optflag(
            "t",
            "",
            "Set additional recipients from message body.\n\
             To:, Cc:, and Bcc: headers will be scanned. \
             Host name of the address is ignored. \
             The Bcc: headers will be deleted from message body.",
        );
        opts.optopt(
            "f",
            "",
            "Set Sender to Return-Path.\n\
                        If From: header absent, From: header is added using Sender.",
            "Sender",
        );
        opts.optflag("", "help", "Print this help menu.");

        let program = args[0].to_string();
        let matches = opts.parse(&args[1..]).unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
            exit(1);
        });

        if matches.opt_present("help") {
            let brief = format!("Usage: {} [Flags] [Recipients]...", program);
            print!("{}", opts.usage(&brief));
            exit(0);
        }

        Self {
            sender: matches.opt_str("f"),
            recipients: matches.free.clone(),
            read_recipients_from_body: matches.opt_present("t"),
        }
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let cmdopt = CmdOption::new_from_cmd_line(args);

    let sender = cmdopt
        .sender
        .unwrap_or("MAILER-DAEMON@localhost".to_string());

    let mut mbox =
        MailMessage::new_from_stream(std::io::stdin()).expect("Failed to read from stream");
    mbox.fix_mail_headers(&sender);

    // TODO spool
    mbox.write_to_mbox(std::io::stdout(), &sender);
}
