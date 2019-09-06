use crate::{config::Config, error::Error, login, update};
use reqwest;
use std::{
    io::{self, prelude::*},
    process,
};

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

pub fn enter_command_mode(
    config: &mut Config,
    client: &reqwest::Client,
) -> Result<(), Error> {
    println!(concat!(
        "Welcome to ",
        crate_name!(),
        "! Type help or ? to get a list of commands.",
    ));

    let mut command_buf = String::with_capacity(0x10);
    let mut children = Vec::with_capacity(2);

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
            Some("help") | Some("?") => {
                help();
                check_children(&mut children)?;
            },
            Some("about") => {
                about();
                check_children(&mut children)?;
            },
            Some("quit") | Some("exit") => {
                check_children(&mut children)?;
                if children.is_empty() {
                    break;
                } else if children.len() == 1 {
                    print!(
                        "Are you sure are you want to exit? There's still a \
                         game instance running! [y/n]\n> ",
                    );
                } else {
                    print!(
                        "Are you sure are you want to exit? There are still \
                         {} game instances running! [y/n]\n> ",
                        children.len(),
                    );
                }
            },
            Some("update") | Some("up") => {
                check_children(&mut children)?;
                if children.is_empty() {
                    update::update(config, client)?
                } else if children.len() == 1 {
                    println!(
                        "There's still a game instance running, can't update \
                         now!",
                    );
                } else {
                    println!(
                        "There are still {} game instances running, can't \
                         update now!",
                        children.len(),
                    );
                }
            },
            Some("login") | Some("play") | Some("launch") => {
                if let Some(c) = login::login(config, client, argv)? {
                    children.push(c);

                    println!("Game launched successfully!");
                }
                check_children(&mut children)?;
            },
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

/// Naive implementation because, let's be real, how many instances of the game
/// are you really going to run concurrently?
fn check_children(
    children: &mut Vec<(String, process::Child)>,
) -> Result<(), Error> {
    let mut i = 0;
    while let Some((username, child)) = children.get_mut(i) {
        if let Some(exit_status) =
            child.try_wait().map_err(Error::ThreadJoinError)?
        {
            if exit_status.success() {
                println!("{}'s instance exited normally.", username);
            } else if let Some(exit_code) = exit_status.code() {
                println!(
                    "{}'s instance exited abnormally. Exit code: {}",
                    username, exit_code,
                );
            } else {
                println!("{}'s instance was killed by a signal.", username);
            }

            children.remove(i);
        } else {
            i += 1;
        }
    }

    Ok(())
}
