use crate::error::Error as AppError;
use fs2::FileExt;
use serde::Deserialize;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;

/// Default config file path. `/etc/sendmail-to-a-spool-file.toml`
const DEFAULT_CONFIG_FILE_PATH: &'static str = concat!("/etc/", env!("CARGO_PKG_NAME"), ".toml");

/// Application configuration
#[derive(PartialEq, Eq, Deserialize, Debug)]
pub struct Config {
    /// Path to spool file.
    pub spool_file: String,
    /// Sender address.
    pub sender: String,
}

impl Config {
    /// Load from a toml file stored in default file path.
    ///
    /// Default file path can be set by environment variable `CONFIG_FILE_PATH` at build process.
    ///
    /// # Errors
    /// File IO error or wrong TOML format (e.g. undefined parameter).
    /// For unix, raise an error if the file or any ancestor is world writable.
    pub fn new_from_default_toml_file() -> Result<Self, AppError> {
        let path = Path::new(option_env!("CONFIG_FILE_PATH").unwrap_or(DEFAULT_CONFIG_FILE_PATH));
        #[cfg(unix)]
        {
            if unix::is_world_writable_ancestors(&path).unwrap_or(false) {
                return Err(AppError::ConfigFileWorldWritable(
                    path.clone().to_path_buf(),
                ));
            }
        }
        Self::new_from_toml_file(&path)
    }

    /// Load from a toml file.
    ///
    /// # Errors
    /// File IO error or wrong TOML format (e.g. undefined parameter).
    pub fn new_from_toml_file<P: AsRef<Path>>(path: P) -> Result<Self, AppError> {
        let path = path.as_ref();
        let toml_str = OpenOptions::new()
            .read(true)
            .open(path)
            .and_then(|mut file| {
                file.lock_shared()?;
                let mut string = String::with_capacity(
                    file.metadata().map(|m| m.len() as usize + 1).unwrap_or(0),
                );
                file.read_to_string(&mut string)?;
                Ok(string.to_owned())
            })
            .map_err(|e| AppError::ConfigIo(path.clone().to_path_buf(), e))?;

        toml::from_str::<Self>(&toml_str)
            .map_err(|e| AppError::ConfigToml(path.clone().to_path_buf(), e))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_config_new_from_toml_file() {
        let tempdir = tempfile::tempdir().unwrap();
        let tempdir_path = tempdir.into_path();

        let config_file_path = tempdir_path.join("config.toml");
        {
            let mut config_file = File::create(&config_file_path).unwrap();
            config_file
                .write_all(
                    "spool_file = \"/var/spool/mail/alice\"\n\
                                sender = \"MAILER-DAEMON@localhost\"\n"
                        .as_bytes(),
                )
                .unwrap();
        }
        assert_eq!(
            Config::new_from_toml_file(&config_file_path).unwrap(),
            Config {
                spool_file: "/var/spool/mail/alice".to_string(),
                sender: "MAILER-DAEMON@localhost".to_string()
            }
        );

        let config_file_path = tempdir_path.join("config2.toml");
        {
            let mut config_file = File::create(&config_file_path).unwrap();
            config_file.write_all("\n".as_bytes()).unwrap();
        }
        assert!(matches!(
            Config::new_from_toml_file(&config_file_path),
            Err(AppError::ConfigToml(_, _))
        ));

        let config_file_path = tempdir_path.join("not-exist-file");
        assert!(matches!(
            Config::new_from_toml_file(&config_file_path),
            Err(AppError::ConfigIo(_, _))
        ));
    }

    #[test]
    fn test_misc_data() {
        assert!(
            Config::new_from_toml_file("misc/sendmail-into-mbox.toml").is_ok()
        );
    }
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::fs;
    use std::io;
    use std::os::unix::fs::PermissionsExt;

    /// Check if the path file and ancestors are world writable.
    pub(super) fn is_world_writable_ancestors(path: &Path) -> Result<bool, io::Error> {
        Ok(is_world_writable(path)?
            || match path.parent() {
                None => false,
                Some(parent) => is_world_writable_ancestors(parent)?,
            })
    }

    /// Check if the path is world writable.
    fn is_world_writable(path: &Path) -> Result<bool, io::Error> {
        Ok(fs::metadata(path)?.permissions().mode() & 0o022 != 0o000)
    }
}
