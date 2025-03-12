#![forbid(unsafe_code)]
#![deny(deprecated)]

mod command;
mod config;
mod error;
mod keyring;
mod login;
mod patch;
mod update;
mod util;

use clap::{
    builder::{ArgPredicate, ValueParser},
    crate_authors, crate_description, crate_name, crate_version, value_parser,
    Arg, ArgAction, Command,
};
use error::Error;
use reqwest::blocking as rb;
use std::{num::NonZeroUsize, process};

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");

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

    let arg_matches = Command::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .aliases(["conf", "configuration"])
                .value_name("CONFIG_FILE")
                .help("Configuration JSON file to use.")
                .long_help(CONFIG_LONG_HELP)
                .num_args(1)
                .value_parser(ValueParser::path_buf())
                .conflicts_with("no-config")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("no-config")
                .long("no-config")
                .aliases(["no-conf", "no-configuration"])
                .help("Don't read nor write any config files.")
                .num_args(0)
                .requires_ifs([
                    (ArgPredicate::IsPresent, "install-dir"),
                    (ArgPredicate::IsPresent, "cache-dir"),
                ])
                .conflicts_with("config")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("install-dir")
                .short('i')
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
                .num_args(1)
                .value_parser(ValueParser::path_buf())
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("cache-dir")
                .short('a')
                .long("cache-dir")
                .value_name("CACHE_DIR")
                .help("Directory for caching game file downloads.")
                .long_help(
                    "Directory for caching game file downloads, which will \
                     be created if it doesn't already exist. Overrides the \
                     value found in the config (if any), but will not be \
                     written to the config. The default cache directory is \
                     named \"cache/\" and is in the same directory as the \
                     config file. Usually you won't need this option.",
                )
                .num_args(1)
                .action(ArgAction::Set)
                .value_parser(ValueParser::path_buf()),
        )
        .arg(
            Arg::new("no-auto-update")
                .short('n')
                .long("no-auto-update")
                .help("Suppress auto-update behavior.")
                .long_help(
                    "Suppresses auto-updating, although you can still decide \
                     to update via the `update`/`up` command.",
                )
                .num_args(0)
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("username")
                .short('u')
                .long("username")
                .help(
                    "Username to immediately login with. This option may be \
                     supplied zero or more times.",
                )
                .long_help(
                    "If this option is supplied, then after (possibly) \
                     auto-updating, the game will be launched with this \
                     username. To login with multiple accounts, specify this \
                     option more than once. The corresponding password(s) \
                     will be prompted for as normal if they aren't saved. \
                     Then, if the login(s) succeed, command mode is entered \
                     (assuming `-d` is not supplied).",
                )
                .num_args(1..)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("detach")
                .short('d')
                .long("detach")
                .help(
                    "Exit after auto-updating (and possibly launching, if \
                     `-u` is supplied).",
                )
                .long_help(
                    "After auto-updating (unless `-n` was supplied), and \
                     after launching the game (if `-u` was supplied), \
                     shticker_book_unwritten will simply exit. If the game \
                     was launched, then its process is thus orphaned. On \
                     POSIX, an orphan continues running normally and is \
                     reparented to the init process (actually, on Linux, the \
                     closest parent process marked as a subreaper).",
                )
                .num_args(0)
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help(
                    "Don't output anything unless necessary or explicitly \
                     requested.",
                )
                .num_args(0)
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("tries")
                .short('t')
                .long("tries")
                .help(
                    "Positive integer number of times to try doing things \
                     involving the network. Defaults to 5.",
                )
                .long_help(
                    "Positive integer number of times to try doing things \
                     that involve interacting with the network. Defaults to \
                     5. Currently works for downloading files, including the \
                     manifest.",
                )
                .num_args(1)
                .action(ArgAction::Set)
                .value_parser(value_parser!(NonZeroUsize)),
        )
        .arg(
            Arg::new("dry-update")
                .short('y')
                .long("dry-update")
                .help(
                    "When auto-updating, only check if updates are available.",
                )
                .long_help(
                    "This flag causes auto-updating to only check whether or \
                     not updates are available, and for which files; that \
                     is, no updates will be downloaded nor applied.",
                )
                .num_args(0)
                .action(ArgAction::SetTrue)
                .conflicts_with("no-auto-update"),
        )
        .get_matches();

    let quiet = arg_matches.get_one("quiet").copied().unwrap_or(false);
    let max_tries =
        if let Some(tries) = arg_matches.get_one::<NonZeroUsize>("tries") {
            *tries
        } else {
            NonZeroUsize::new(5).unwrap()
        };

    let (mut config, config_path) = config::get_config(
        arg_matches.get_one("no-config").copied().unwrap_or(false),
        arg_matches.get_one("config").cloned(),
        arg_matches.get_one("install-dir").cloned(),
        arg_matches.get_one("cache-dir").cloned(),
        quiet,
    )?;

    let client = rb::ClientBuilder::new()
        .build()
        .map_err(Error::HttpClientCreate)?;

    if !arg_matches
        .get_one("no-auto-update")
        .copied()
        .unwrap_or(false)
    {
        update::update(
            &config,
            &client,
            quiet,
            max_tries,
            arg_matches.get_one("dry-update").copied().unwrap_or(false),
        )?;

        if !quiet {
            println!();
        }
    }

    command::enter_command_mode(
        &mut config,
        &config_path,
        &client,
        quiet,
        arg_matches
            .get_many::<String>("username")
            .map(|it| it.map(|v| v.as_str())),
        arg_matches.get_one("detach").copied().unwrap_or(false),
        max_tries,
    )
}
