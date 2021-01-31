# sendmail-into-mbox

This project is currently under beta stage of development.

Alternative sendmail command to read email from stdin and write to a mbox file directly.

![Rust](https://github.com/cat-in-136/sendmail-into-mbox/workflows/Rust/badge.svg)

Typical E-Mail delivery process is that the message is transferred from sender MUA by one or multiple MTA
until it reaches the MDA at the destination server.
The receiver fetches the message on the destination server using their MUA.

    Sender -- MUA -(SMTP)-> MTA --> MDA -(POP/IMAP)-> MUA -- Receiver

It is too complex for single user purpose.
sendmail-into-mbox stores all the mails directly into a local file in mbox format without going through any server.

It simulates the old-fashion behavior of communicate with a single user in your PC as sendmail.
sendmail-into-mbox ignores any command-line arguments, so it is thought to be compatible with sendmail command.
The recipients such as specified at `To:`, `Cc:`, `Bcc:` and command-line arguments are also ignored,
and the file is always saved to a specific file.
If you set the save destination to be the same as `$MAIL` (e.g. `/var/spool/mail/username`),
you can read the mails with your old-fashioned MUA such as mailx, biff, etc... that can handles mbox file.

    You -- sendmail-into-mbox --> /var/spool/mail/username --> MUA -- You

## Install

### RPM Package

RPM package is available on the release page.

The RPM package take care of using `/usr/sbin/alternatives` to ensure that only one
`/usr/sbin/sendmail` is set as the system default at a time.
`sendmail-into-mbox` is configured as low priority (10).

### Manual Install

```
# install -m 2755 -o root -g mail \
    target/release/sendmail-into-mbox /usr/sbin
# ln -s /usr/sbin/sendmail-into-mbox /usr/sbin/sendmail
# install -m 0655 -o root -g root \
    misc/sendmail-into-mbox.toml /etc
```

## Settings

To configure the behavior of sendmail-into-mbox, edit `/etc/sendmail-into-mbox.toml`.
Example setting is located at `misc/sendmail-into-mbox.toml`.

In typical use case, change the value of `spool_file` from `misc/sendmail-into-mbox.toml` to the value of your `$MAIL`.

```toml
spool_file = "/var/spool/mail/alice"
```

sendmail-into-mbox does not create this file, so you need to create it in advance.

```
# touch /var/spool/mail/alice
# chmod 660 /var/spool/mail/alice
# chown alice:mail /var/spool/mail/alice
```

## Build

Dependencies:

* Rust 1.42+

To build, just run `cargo build`. To run, just run `cargo run`.


## License

MIT License. See the LICENSE file.
