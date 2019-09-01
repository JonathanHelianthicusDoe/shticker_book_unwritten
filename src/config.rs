use crate::error::Error;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

pub const DEFAULT_MANIFEST_URI: &str =
    "https://cdn.toontownrewritten.com/content/patchmanifest.txt";
pub const DEFAULT_CDN_URI: &str =
    "https://s3.amazonaws.com/download.toontownrewritten.com/patches/";

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub install_dir:     PathBuf,
    pub cache_dir:       PathBuf,
    pub manifest_uri:    String,
    pub cdn_uri:         String,
    pub store_passwords: bool,
    pub accounts:        Vec<Account>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Account {
    pub username: String,
    pub password: Option<String>,
}

pub fn get_config(config_path: Option<&str>) -> Result<Config, Error> {
    let config_path = if let Some(s) = config_path {
        PathBuf::from(s)
    } else {
        let mut xdg_config_home = String::new();
        let mut home = String::new();

        for (key, value) in env::vars() {
            match key.as_str() {
                "XDG_CONFIG_HOME" => xdg_config_home = value,
                "HOME" => home = value,
                _ =>
                    if !(home.is_empty() || xdg_config_home.is_empty()) {
                        break;
                    },
            }
        }

        if !xdg_config_home.is_empty() {
            [xdg_config_home.as_str(), crate_name!(), "config.json"]
                .iter()
                .collect()
        } else if !home.is_empty() {
            [home.as_str(), ".config", crate_name!(), "config.json"]
                .iter()
                .collect()
        } else {
            return Err(Error::NoPossibleConfigPath);
        }
    };

    match File::open(&config_path) {
        Ok(f) => serde_json::from_reader(f).map_err(Error::DeserializeError),
        Err(ioe) =>
            match ioe.kind() {
                io::ErrorKind::NotFound => {
                    fs::create_dir_all(config_path.parent().ok_or_else(
                        || Error::BadConfigPath(config_path.clone()),
                    )?)
                    .map_err(Error::MkdirFailure)?;

                    let mut new_config_file = File::create(&config_path)
                        .map_err(|ioe| match ioe.kind() {
                            io::ErrorKind::PermissionDenied =>
                                Error::PermissionDenied(ioe),
                            _ => Error::UnknownIoError(ioe),
                        })?;
                    let new_config = prompt_for_config_values(&config_path)?;

                    serde_json::to_writer_pretty(
                        &mut new_config_file,
                        &new_config,
                    )
                    .map_err(Error::SerializeError)?;

                    Ok(new_config)
                },
                io::ErrorKind::PermissionDenied =>
                    Err(Error::PermissionDenied(ioe)),
                _ => Err(Error::UnknownIoError(ioe)),
            },
    }
}

fn prompt_for_config_values<P: AsRef<Path>>(
    config_path: P,
) -> Result<Config, Error> {
    print!(
        "No configuration file was found at {}\nAnswer a few prompts, and a \
         new config file will be created there.\n\nFully qualified path to \
         desired TTR installation directory\n(will be created if it doesn't \
         already exist):\n> ",
        config_path.as_ref().display(),
    );
    io::stdout().flush().map_err(Error::StdoutError)?;
    let mut install_dir = String::with_capacity(0x30);
    io::stdin()
        .read_line(&mut install_dir)
        .map_err(Error::StdinError)?;

    print!(
        "\nDo you want passwords for your accounts to be stored in the \
         config file? [yes/no]\nThe passwords will be stored IN PLAIN TEXT, \
         so if you want your passwords to be managed without storing them on \
         your hard drive in plain text, you will have to use a separate \
         password manager app:\n> "
    );
    io::stdout().flush().map_err(Error::StdoutError)?;
    let mut yes_no = String::with_capacity(4);
    io::stdin()
        .read_line(&mut yes_no)
        .map_err(Error::StdinError)?;
    yes_no.make_ascii_lowercase();
    loop {
        let yes_no_trimmed = yes_no.as_str().trim();
        if yes_no_trimmed == "yes" || yes_no_trimmed == "no" {
            println!();

            return Ok(Config {
                install_dir:     PathBuf::from(install_dir.trim()),
                cache_dir:       config_path
                    .as_ref()
                    .parent()
                    .ok_or_else(|| {
                        Error::BadConfigPath(config_path.as_ref().to_owned())
                    })?
                    .join("cache"),
                manifest_uri:    DEFAULT_MANIFEST_URI.to_owned(),
                cdn_uri:         DEFAULT_CDN_URI.to_owned(),
                store_passwords: yes_no_trimmed == "yes",
                accounts:        Vec::new(),
            });
        }

        print!("Please enter \"yes\" or \"no\" (without quotes):\n> ");
        io::stdout().flush().map_err(Error::StdoutError)?;
        yes_no.clear();
        io::stdin()
            .read_line(&mut yes_no)
            .map_err(Error::StdinError)?;
        yes_no.make_ascii_lowercase();
    }
}
