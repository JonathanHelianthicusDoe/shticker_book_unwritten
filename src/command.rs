use crate::{config::Config, error::Error, login, update};
use clap::{crate_name, crate_version};
use reqwest;
use std::{
    io::{self, prelude::*},
    path::Path,
    process,
    time,
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
                                     instance is specified by its PID or by
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

pub fn enter_command_mode<'a, P: AsRef<Path>, U: Iterator<Item = &'a str>>(
    config: &mut Config,
    config_path: P,
    client: &reqwest::Client,
    quiet: bool,
    maybe_usernames: Option<U>,
    detach: bool,
) -> Result<(), Error> {
    let mut children = Vec::new();
    if let Some(usernames) = maybe_usernames {
        for username in usernames {
            if let Some(c) = login::login(
                config,
                &config_path,
                client,
                quiet,
                [username].iter().copied(),
            )? {
                if !detach {
                    children.push(c);
                }

                if !quiet {
                    println!("Game launched successfully!");
                }
            }
        }

        if !detach && !quiet {
            println!();
        }
    };

    if detach {
        return Ok(());
    }

    if !quiet {
        println!(concat!(
            "Welcome to ",
            crate_name!(),
            "! Type help or ? to get a list of commands.",
        ));
    }
    let mut command_buf = String::with_capacity(0x10);

    'outer: loop {
        print!("> ");
        io::stdout().flush().map_err(Error::StdoutError)?;
        command_buf.clear();
        io::stdin()
            .read_line(&mut command_buf)
            .map_err(Error::StdinError)?;

        // ^D
        if command_buf.is_empty() {
            println!();
            command_buf.push_str("quit");
        }

        let mut argv = command_buf
            .split(char::is_whitespace)
            .filter(|arg| !arg.is_empty());
        match argv.next() {
            None => (),
            Some("help") | Some("?") => {
                help();
                check_children(quiet, &mut children)?;
            },
            Some("about") => {
                about();
                check_children(quiet, &mut children)?;
            },
            Some("quit") | Some("exit") => {
                check_children(quiet, &mut children)?;
                if children.is_empty() {
                    break;
                } else if children.len() == 1 {
                    print!(
                        "Are you sure are you want to exit? There's still a \
                         game instance running. [y/n]\n> ",
                    );
                } else {
                    print!(
                        "Are you sure are you want to exit? There are still \
                         {} game instances running. [y/n]\n> ",
                        children.len(),
                    );
                }

                io::stdout().flush().map_err(Error::StdoutError)?;
                command_buf.clear();
                io::stdin()
                    .read_line(&mut command_buf)
                    .map_err(Error::StdinError)?;

                loop {
                    match command_buf.trim_start().as_bytes().get(0) {
                        Some(b'y') | Some(b'Y') => break 'outer,
                        Some(b'n') | Some(b'N') => break,
                        _ => (),
                    }

                    print!("[y/n]?\n> ");
                    io::stdout().flush().map_err(Error::StdoutError)?;
                    command_buf.clear();
                    io::stdin()
                        .read_line(&mut command_buf)
                        .map_err(Error::StdinError)?;
                }
            },
            Some("update") | Some("up") => {
                check_children(quiet, &mut children)?;
                if children.is_empty() {
                    update::update(config, client, quiet)?
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
                if let Some(c) =
                    login::login(config, &config_path, client, quiet, argv)?
                {
                    children.push(c);

                    if !quiet {
                        println!("Game launched successfully!");
                    }
                }
                check_children(quiet, &mut children)?;
            },
            Some("instances") | Some("running") => {
                check_children(quiet, &mut children)?;
                display_instances(&children);
            },
            Some("kill") | Some("close") => {
                check_children(quiet, &mut children)?;
                kill_instance(quiet, &mut children, argv.next())?;
            },
            Some("accounts") | Some("logins") => {
                check_children(quiet, &mut children)?;
                display_accounts(config, &children)?;
            },
            _ => {
                check_children(quiet, &mut children)?;
                println!(
                    "Unrecognized command. Type help or ? to get a list of \
                     commands.",
                );
            },
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

fn display_instances(instances: &[(String, process::Child, time::Instant)]) {
    if instances.is_empty() {
        return;
    }

    fn count_decimal_digits(n: u32) -> usize {
        if n >= 100_000 {
            if n >= 10_000_000 {
                if n >= 1_000_000_000 {
                    10
                } else if n >= 100_000_000 {
                    9
                } else {
                    8
                }
            } else if n >= 1_000_000 {
                7
            } else {
                6
            }
        } else if n >= 1_000 {
            if n >= 10_000 {
                5
            } else {
                4
            }
        } else if n >= 100 {
            3
        } else if n >= 10 {
            2
        } else {
            1
        }
    }

    let (max_name_len, max_pid_len) = instances.iter().fold(
        ("username".len(), "pid".len()),
        |(max_name_len, max_pid_len), (name, child, _)| {
            (
                max_name_len.max(name.len()),
                max_pid_len.max(count_decimal_digits(child.id())),
            )
        },
    );

    print!("username ");
    for _ in 0..max_name_len.saturating_sub("username".len()) {
        print!(" ");
    }
    print!("| pid ");
    for _ in 0..max_pid_len.saturating_sub("pid".len()) {
        print!(" ");
    }
    print!("| uptime\n---------");
    for _ in 0..max_name_len.saturating_sub("username".len()) {
        print!("-");
    }
    print!("+-----");
    for _ in 0..max_pid_len.saturating_sub("pid".len()) {
        print!("-");
    }
    println!("+-----------");
    for (name, child, timestamp) in instances {
        let pid = child.id();

        print!("{} ", name);
        for _ in 0..max_name_len - name.len() {
            print!(" ");
        }

        print!("| {} ", pid);
        for _ in 0..max_pid_len - count_decimal_digits(pid) {
            print!(" ");
        }

        let uptime_sec = timestamp.elapsed().as_secs();
        let secs = uptime_sec % 60;
        let minutes = (uptime_sec / 60) % 60;
        let hours = uptime_sec / (60 * 60);
        println!("| {}h {:02}m {:02}s", hours, minutes, secs);
    }
}

fn kill_instance(
    quiet: bool,
    children: &mut Vec<(String, process::Child, time::Instant)>,
    arg: Option<&str>,
) -> Result<(), Error> {
    let instance_str = if let Some(s) = arg {
        s
    } else {
        println!("Expected the <instance> argument!");

        return Ok(());
    };
    let maybe_instance_int: Option<u32> = instance_str.parse().ok();

    let maybe_instance = if let Some(pid) = maybe_instance_int {
        if let Some(c) = children
            .iter_mut()
            .enumerate()
            .find(|(_, (_, child, _))| child.id() == pid)
        {
            Some(c)
        } else {
            children
                .iter_mut()
                .enumerate()
                .find(|(_, (name, _, _))| name == instance_str)
        }
    } else {
        children
            .iter_mut()
            .enumerate()
            .find(|(_, (name, _, _))| name == instance_str)
    };

    if let Some((i, (name, child, timestamp))) = maybe_instance {
        let pid = child.id();
        let uptime_sec = timestamp.elapsed().as_secs();

        if !quiet {
            println!("Killing instance...");
        }

        if let Err(ioe) = child.kill() {
            if ioe.kind() != io::ErrorKind::InvalidInput {
                return Err(Error::ProcessKillError(ioe));
            }
        }

        if !quiet {
            println!("Joining instance's thread...");
        }

        child.wait().map_err(Error::ThreadJoinError)?;

        if !quiet {
            println!(
                "Successfully killed {}'s instance with pid {},",
                name, pid
            );
            let secs = uptime_sec % 60;
            let minutes = (uptime_sec / 60) % 60;
            let hours = uptime_sec / (60 * 60);
            println!(
                "which had an approximate uptime of {}h {:02}m {:02}s.",
                hours, minutes, secs,
            );
        }

        children.remove(i);
    } else {
        println!("No currently running instances have that username or pid.");
    }

    Ok(())
}

fn display_accounts(
    config: &Config,
    children: &[(String, process::Child, time::Instant)],
) -> Result<(), Error> {
    let max_name_len = if let Some(l) =
        config.accounts.iter().map(|(un, _)| un.len()).max()
    {
        l
    } else {
        return Ok(());
    };

    for (username, saved_password) in
        config.accounts.iter().map(|(un, p)| (un, p.is_string()))
    {
        print!(
            "{} {}   ",
            if children.iter().any(|(un, _, _)| un == username) {
                '*'
            } else {
                ' '
            },
            username,
        );
        for _ in 0..max_name_len - username.len() {
            print!(" ");
        }
        println!("Password?: {}", if saved_password { "yes" } else { "no" });
    }

    Ok(())
}

/// Naïve implementation because, let's be real, how many instances of the game
/// are you really going to run concurrently?
fn check_children(
    quiet: bool,
    children: &mut Vec<(String, process::Child, time::Instant)>,
) -> Result<(), Error> {
    let mut i = 0;
    while let Some((username, child, _)) = children.get_mut(i) {
        if let Some(exit_status) =
            child.try_wait().map_err(Error::ThreadJoinError)?
        {
            if !quiet {
                if exit_status.success() {
                    println!("{}'s instance exited normally.", username);
                } else if let Some(exit_code) = exit_status.code() {
                    println!(
                        "{}'s instance exited abnormally. Exit code: {}",
                        username, exit_code,
                    );
                } else {
                    println!(
                        "{}'s instance was killed by a signal.",
                        username,
                    );
                }
            }

            children.remove(i);
        } else {
            i += 1;
        }
    }

    Ok(())
}
