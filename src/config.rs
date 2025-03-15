use crate::{error::Error, util};
use clap::crate_name;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

const DEFAULT_MANIFEST_URI: &str =
    "https://cdn.toontownrewritten.com/content/patchmanifest.txt";
const DEFAULT_CDN_URI: &str =
    "https://download.toontownrewritten.com/patches/";

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub install_dir: PathBuf,
    /// TODO: Implement a way to clear the cache, presumably automatically
    pub cache_dir: PathBuf,
    pub manifest_uri: String,
    pub cdn_uri: String,
    pub store_passwords: bool,
    pub accounts: serde_json::Map<String, serde_json::Value>,
}

impl Config {
    /// Same return type as `BTreeMap::insert`.
    #[cfg(not(all(target_os = "linux", feature = "secret-store")))]
    pub fn add_account(
        &mut self,
        username: String,
        password: String,
    ) -> Option<serde_json::Value> {
        if self.store_passwords {
            self.accounts
                .insert(username, serde_json::Value::String(password))
        } else {
            self.accounts.insert(username, serde_json::Value::Null)
        }
    }

    pub fn forget_account(&mut self, username: &str) {
        self.accounts.remove(username);
    }
}

pub fn get_config(
    no_config: bool,
    config_path: Option<PathBuf>,
    install_path: Option<PathBuf>,
    cache_path: Option<PathBuf>,
    quiet: bool,
) -> Result<(Config, PathBuf), Error> {
    let inject_arg_values = |c| {
        let c = if let Some(ip) = install_path.clone() {
            Config {
                install_dir: ip,
                ..c
            }
        } else {
            c
        };

        if let Some(cp) = cache_path.clone() {
            Config { cache_dir: cp, ..c }
        } else {
            c
        }
    };

    if !no_config {
        let config_path = if let Some(s) = config_path {
            s
        } else {
            #[cfg(target_os = "linux")]
            {
                let mut xdg_config_home = String::new();
                let mut home = String::new();

                for (key, value) in env::vars() {
                    match key.as_str() {
                        "XDG_CONFIG_HOME" => xdg_config_home = value,
                        "HOME" => home = value,
                        _ => {
                            if !(home.is_empty() || xdg_config_home.is_empty())
                            {
                                break;
                            }
                        }
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
            }
            #[cfg(windows)]
            {
                let mut appdata = String::new();

                for (key, value) in env::vars() {
                    match key.as_str() {
                        "APPDATA" => appdata = value,
                        _ => {
                            if !appdata.is_empty() {
                                break;
                            }
                        }
                    }
                }

                if !appdata.is_empty() {
                    [appdata.as_str(), crate_name!(), "config.json"]
                        .iter()
                        .collect()
                } else {
                    return Err(Error::NoPossibleConfigPath);
                }
            }
            #[cfg(target_os = "macos")]
            {
                let mut home = String::new();

                for (key, value) in env::vars() {
                    match key.as_str() {
                        "HOME" => home = value,
                        _ => {
                            if !(home.is_empty()) {
                                break;
                            }
                        }
                    }
                }

                if !home.is_empty() {
                    [
                        home.as_str(),
                        "Library",
                        "Preferences",
                        crate_name!(),
                        "config.json",
                    ]
                    .iter()
                    .collect()
                } else {
                    return Err(Error::NoPossibleConfigPath);
                }
            }
        };

        if !quiet {
            println!("Using {} as the config path...", config_path.display());
        }

        match File::open(&config_path) {
            Ok(f) => serde_json::from_reader(f)
                .map_err(Error::Deserialize)
                .map(|c| (inject_arg_values(c), config_path)),
            Err(ioe) => match ioe.kind() {
                io::ErrorKind::NotFound => {
                    let config_dir =
                        config_path.parent().ok_or_else(|| {
                            Error::BadConfigPath(config_path.clone())
                        })?;
                    fs::create_dir_all(config_dir).map_err(|ioe| {
                        Error::Mkdir(config_dir.to_path_buf(), ioe)
                    })?;

                    let mut new_config_file = util::create_file(&config_path)?;
                    let new_config = prompt_for_config_values(&config_path)?;

                    serde_json::to_writer_pretty(
                        &mut new_config_file,
                        &new_config,
                    )
                    .map_err(Error::Serialize)?;

                    Ok((inject_arg_values(new_config), config_path))
                }
                io::ErrorKind::PermissionDenied => {
                    Err(Error::PermissionDenied(
                        format!("opening {:?}", config_path),
                        ioe,
                    ))
                }
                _ => Err(Error::UnknownIo(
                    format!("opening {:?}", config_path),
                    ioe,
                )),
            },
        }
    } else {
        if !quiet {
            println!("Not using any config file...");
        }

        Ok((
            Config {
                install_dir: install_path.ok_or_else(|| {
                    Error::MissingCommandLineArg("--install-dir")
                })?,
                cache_dir: cache_path.ok_or_else(|| {
                    Error::MissingCommandLineArg("--cache-dir")
                })?,
                manifest_uri: DEFAULT_MANIFEST_URI.to_owned(),
                cdn_uri: DEFAULT_CDN_URI.to_owned(),
                store_passwords: false,
                accounts: serde_json::Map::default(),
            },
            PathBuf::new(),
        ))
    }
}

fn prompt_for_config_values<P: AsRef<Path>>(
    config_path: P,
) -> Result<Config, Error> {
    print!(
        "No configuration file was found at {}\nAnswer a few prompts, and a \
         new config file will be created there.\n\nFully-qualified path to \
         desired installation directory\n(will be created if it doesn't \
         already exist):\n> ",
        config_path.as_ref().display(),
    );
    io::stdout().flush().map_err(Error::Stdout)?;
    let mut install_dir = String::with_capacity(0x30);
    io::stdin()
        .read_line(&mut install_dir)
        .map_err(Error::Stdin)?;

    #[cfg(not(all(target_os = "linux", feature = "secret-store")))]
    print!(
        "\nDo you want passwords for your accounts to be stored in the \
         config file? <yes | no>\nThe passwords will be stored IN PLAIN TEXT, \
         so if you want your passwords to be managed without storing them on \
         your filesystem in plain text, then you'll have to use a separate \
         password manager app:\n> "
    );
    #[cfg(all(target_os = "linux", feature = "secret-store"))]
    print!(
        "\nDo you want passwords for your accounts to be stored on your \
         default Secret Service keyring? <yes | no>\n> "
    );
    io::stdout().flush().map_err(Error::Stdout)?;
    let mut yes_no = String::with_capacity(4);
    io::stdin().read_line(&mut yes_no).map_err(Error::Stdin)?;
    yes_no.make_ascii_lowercase();
    loop {
        let yes_no_trimmed = yes_no.trim();
        if yes_no_trimmed == "yes" || yes_no_trimmed == "no" {
            println!();

            return Ok(Config {
                install_dir: PathBuf::from(install_dir.trim()),
                cache_dir: config_path
                    .as_ref()
                    .parent()
                    .ok_or_else(|| {
                        Error::BadConfigPath(config_path.as_ref().to_owned())
                    })?
                    .join("cache"),
                manifest_uri: DEFAULT_MANIFEST_URI.to_owned(),
                cdn_uri: DEFAULT_CDN_URI.to_owned(),
                store_passwords: yes_no_trimmed == "yes",
                accounts: serde_json::Map::default(),
            });
        }

        print!("Please enter yes or no:\n> ");
        io::stdout().flush().map_err(Error::Stdout)?;
        yes_no.clear();
        io::stdin().read_line(&mut yes_no).map_err(Error::Stdin)?;
        yes_no.make_ascii_lowercase();
    }
}

pub fn commit_config<P: AsRef<Path>>(
    config: &Config,
    config_path: P,
) -> Result<(), Error> {
    let temp_config_path = config_path
        .as_ref()
        .parent()
        .ok_or(Error::NoPossibleConfigPath)?
        .join(".config.json.temp");
    let mut temp_config_file = util::create_file(&temp_config_path)?;

    serde_json::to_writer_pretty(&mut temp_config_file, config)
        .map_err(Error::Serialize)?;
    temp_config_file
        .write_all(b"\n")
        .map_err(|e| Error::FileWrite(temp_config_path.clone(), e))?;
    fs::rename(&temp_config_path, config_path.as_ref()).map_err(|_| {
        Error::FileRename(temp_config_path, config_path.as_ref().to_owned())
    })
}
