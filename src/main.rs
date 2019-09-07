#![deny(clippy::all)]

#[macro_use]
extern crate clap;
extern crate bzip2;
extern crate reqwest;
extern crate rpassword;
extern crate serde;
extern crate serde_json;
extern crate sha1;

mod command;
mod config;
mod error;
mod login;
mod patch;
mod update;

use clap::{App, Arg};
use error::Error;
use reqwest::ClientBuilder;
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);

        process::exit(e.return_code())
    }
}

fn run() -> Result<(), Error> {
    let arg_matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .aliases(&["conf", "configuration"])
                .value_name("CONFIG_FILE")
                .help("Configuration JSON file to use")
                .long_help(concat!(
                    "Configuration JSON file to use. Defaults to \
                     \"$XDG_CONFIG_HOME\"/",
                    crate_name!(),
                    "/config.json and then to \"$HOME\"/.config/",
                    crate_name!(),
                    "/config.json",
                ))
                .takes_value(true)
                .conflicts_with("no-config"),
        )
        .arg(
            Arg::with_name("no-config")
                .long("no-config")
                .aliases(&["no-conf", "no-configuration"])
                .help("Don't read or write any config files")
                .takes_value(false)
                .requires_all(&["install-dir", "cache-dir"])
                .conflicts_with("config"),
        )
        .arg(
            Arg::with_name("install-dir")
                .short("i")
                .long("install-dir")
                .value_name("INSTALL_DIR")
                .help("Directory of TTR installation")
                .long_help(
                    "The directory of the TTR installation, which will be \
                     automatically created if it doesn't already exist. \
                     Overrides the value found in the config (if any), but \
                     will not be written to the config. Usually you won't \
                     need this option.",
                )
                .takes_value(true),
        )
        .arg(
            Arg::with_name("cache-dir")
                .short("a")
                .long("cache-dir")
                .value_name("CACHE_DIR")
                .help("Directory for caching game file downloads")
                .long_help(
                    "Directory for caching game file downloads, which will \
                     be created if it doesn't already exist. Overrides the \
                     value found in the config (if any), but will not be \
                     written to the config. The default cache directory is \
                     named \"cache\" and is in the same directory as the \
                     config file. Usually you won't need this option.",
                )
                .takes_value(true),
        )
        .arg(
            Arg::with_name("no-auto-update")
                .short("n")
                .long("no-auto-update")
                .help("Suppress auto-update behavior")
                .long_help(
                    "Suppresses auto-updating, although you can still decide \
                     to update via the \"update\"/\"up\" command.",
                )
                .takes_value(false),
        )
        .get_matches();

    let (mut config, config_path) = config::get_config(
        arg_matches.is_present("no-config"),
        arg_matches.value_of("CONFIG_FILE"),
        arg_matches.value_of("INSTALL_DIR"),
        arg_matches.value_of("CACHE_DIR"),
    )?;

    let client = ClientBuilder::new()
        .build()
        .map_err(Error::HttpClientCreateError)?;

    if !arg_matches.is_present("no-auto-update") {
        update::update(&config, &client)?;

        println!();
    }

    command::enter_command_mode(&mut config, &config_path, &client)?;

    if !arg_matches.is_present("no-config") {
        config::commit_config(&config, &config_path)?;
    }

    Ok(())
}
