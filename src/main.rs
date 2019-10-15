#![forbid(unsafe_code)]
#![deny(clippy::all)]

mod command;
mod config;
mod error;
mod login;
mod patch;
mod update;
mod util;

use clap::{
    crate_authors,
    crate_description,
    crate_name,
    crate_version,
    App,
    Arg,
};
use error::Error;
use reqwest::blocking as rb;
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);

        process::exit(e.return_code())
    }
}

fn run() -> Result<(), Error> {
    #[cfg(target_os = "linux")]
    const CONFIG_LONG_HELP: &str = concat!(
        "Configuration JSON file to use. Defaults to \"$XDG_CONFIG_HOME\"/",
        crate_name!(),
        "/config.json and then to \"$HOME\"/.config/",
        crate_name!(),
        "/config.json",
    );
    #[cfg(windows)]
    const CONFIG_LONG_HELP: &str = concat!(
        r"Configuration JSON file to use. Defaults to %APPDATA%\",
        crate_name!(),
        r"\config.json",
    );
    #[cfg(target_os = "macos")]
    const CONFIG_LONG_HELP: &str = concat!(
        "Configuration JSON file to use. Defaults to ",
        "$HOME/Library/Preferences/",
        crate_name!(),
        "/config.json",
    );

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
                .help("Configuration JSON file to use.")
                .long_help(CONFIG_LONG_HELP)
                .takes_value(true)
                .conflicts_with("no-config"),
        )
        .arg(
            Arg::with_name("no-config")
                .long("no-config")
                .aliases(&["no-conf", "no-configuration"])
                .help("Don't read or write any config files.")
                .takes_value(false)
                .requires_all(&["install-dir", "cache-dir"])
                .conflicts_with("config"),
        )
        .arg(
            Arg::with_name("install-dir")
                .short("i")
                .long("install-dir")
                .value_name("INSTALL_DIR")
                .help("Directory of TTR installation.")
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
                .help("Directory for caching game file downloads.")
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
                .help("Suppress auto-update behavior.")
                .long_help(
                    "Suppresses auto-updating, although you can still decide \
                     to update via the \"update\"/\"up\" command.",
                )
                .takes_value(false),
        )
        .arg(
            Arg::with_name("username")
                .short("u")
                .long("username")
                .help("Username(s) to immediately login with.")
                .long_help(
                    "If this option is supplied, then after (possibly) \
                     auto-updating, the game will be launched with these \
                     username(s). The password(s) will be prompted for as \
                     normal if they aren't saved. Then, if the login(s) \
                     succeed, command mode is entered (assuming -d is not \
                     supplied).",
                )
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("detach")
                .short("d")
                .long("detach")
                .help(
                    "Exit after auto-updating (and possibly launching, if -u \
                     is supplied).",
                )
                .long_help(
                    "After auto-updating (unless -n was supplied), and after \
                     launching the game (if -u was supplied), \
                     shticker_book_unwritten will simply exit. If the game \
                     was launched, then its process is thus orphaned. On \
                     POSIX, an orphan continues running normally and is \
                     reparented to the init process (actually, on Linux, the \
                     closest parent process marked as a subreaper).",
                )
                .takes_value(false),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .help(
                    "Don't output anything unless necessary or explicitly \
                     requested.",
                )
                .takes_value(false),
        )
        .get_matches();

    let quiet = arg_matches.is_present("quiet");

    let (mut config, config_path) = config::get_config(
        arg_matches.is_present("no-config"),
        arg_matches.value_of("config"),
        arg_matches.value_of("install-dir"),
        arg_matches.value_of("cache-dir"),
        quiet,
    )?;

    let client = rb::ClientBuilder::new()
        .build()
        .map_err(Error::HttpClientCreateError)?;

    if !arg_matches.is_present("no-auto-update") {
        update::update(&config, &client, quiet)?;

        if !quiet {
            println!();
        }
    }

    command::enter_command_mode(
        &mut config,
        &config_path,
        &client,
        quiet,
        arg_matches.values_of("username"),
        arg_matches.is_present("detach"),
    )
}
