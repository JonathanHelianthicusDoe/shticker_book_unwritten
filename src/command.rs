use crate::{config::Config, error::Error, update};
use std::io::{self, prelude::*};

enum Command {
    Help,
    About,
    Quit,
    Update,
    Login,
    ViewInstances,
    KillInstance,
    SavedLogins,
}

const HELP_TEXT: &str = "\
Commands
========
  help, ?                          Display this help text.
  about                            Display info about this program.
  quit, exit                       Quit this program.
  update, up                       Update the game files.
  login, play, launch              Launch TTR. Specify -n or --no-save to not
    [username] [-n, --no-save]       save this login, even if it's successful.
  instances, running               List currently running TTR instances.
  kill, close <instance>           Forcibly close a running TTR instance. The
                                     instance is specified by its number or by
                                     its username.
  accounts, logins                 List all saved accounts/logins.
";
const ABOUT_TEXT: &str = concat!(
    crate_name!(),
    " v",
    crate_version!(),
    "\nLicensed under the GNU AGPL v3+. Source available at\n<",
    env!("CARGO_PKG_REPOSITORY"),
    ">\n",
);

pub fn enter_command_mode(config: &Config) -> Result<(), Error> {
    println!(concat!(
        "Welcome to ",
        crate_name!(),
        "! Type help or ? to get a list of commands.",
    ));

    let mut command_buf = String::with_capacity(0x10);
    loop {
        print!("> ");
        io::stdout().flush().map_err(Error::StdoutError)?;
        command_buf.clear();
        io::stdin()
            .read_line(&mut command_buf)
            .map_err(Error::StdinError)?;

        // ^D
        if command_buf.is_empty() {
            println!();

            break;
        }

        let mut argv = command_buf
            .split(char::is_whitespace)
            .filter(|arg| !arg.is_empty());
        match argv.next() {
            None => (),
            Some("help") | Some("?") => help(),
            Some("about") => about(),
            Some("quit") | Some("exit") => break,
            Some("update") | Some("up") => update::update(config)?,
            Some("login") | Some("play") | Some("launch") => unimplemented!(),
            Some("instances") | Some("running") => unimplemented!(),
            Some("kill") | Some("close") => unimplemented!(),
            Some("accounts") | Some("logins") => unimplemented!(),
            _ => println!(
                "Unrecognized command. Type help or ? to get a list of \
                 commands.",
            ),
        }
    }

    Ok(())
}

fn help() {
    print!("{}", HELP_TEXT);
}

fn about() {
    print!("{}", ABOUT_TEXT);
}
