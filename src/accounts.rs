//! Subcommands for `accounts`/`logins`

use crate::{
    config::{Config, commit_config},
    error::Error,
};
use std::{
    io::{self, Write},
    path::Path,
};

#[cfg(not(all(target_os = "linux", feature = "secret-store")))]
fn account_exists(config: &Config, username: &str) -> Result<bool, Error> {
    Ok(config.accounts.contains_key(username))
}

#[cfg(all(target_os = "linux", feature = "secret-store"))]
fn account_exists(config: &Config, username: &str) -> Result<bool, Error> {
    Ok(config.accounts.contains_key(username)
        || crate::keyring::account_exists(username)?)
}

pub(crate) fn forget_account<P: AsRef<Path>>(
    config: &mut Config,
    config_path: P,
    quiet: bool,
    maybe_username: Option<&str>,
) -> Result<(), Error> {
    let mut username_buf;
    let username = if let Some(u) = maybe_username {
        u
    } else {
        username_buf = String::with_capacity(0x10);
        print!("Enter the username of the account to forget:\n> ");
        io::stdout().flush().map_err(Error::Stdout)?;
        io::stdin()
            .read_line(&mut username_buf)
            .map_err(Error::Stdin)?;

        &username_buf
    };

    if !account_exists(config, username)? {
        println!("No account with that username was found.");

        return Ok(());
    }

    let mut confirm_buf = String::with_capacity(4);
    #[cfg(not(all(target_os = "linux", feature = "secret-store")))]
    print!(
        "Are you absolutely certain that you want to forget the account with \
         username\n{username}?\nThis will ERASE the username & its associated \
         password from\nyour config file.\n\
         Type yes to confirm:\n> "
    );
    #[cfg(all(target_os = "linux", feature = "secret-store"))]
    print!(
        "Are you absolutely certain that you want to forget the account with \
         username\n{username}?\nThis will ERASE the username & its associated \
         password from\nboth your config file & your Secret Service keyring.\n\
         Type yes to confirm:\n> "
    );
    io::stdout().flush().map_err(Error::Stdout)?;
    io::stdin()
        .read_line(&mut confirm_buf)
        .map_err(Error::Stdin)?;
    confirm_buf.make_ascii_lowercase();
    if confirm_buf.trim() != "yes" {
        if !quiet {
            println!("No accounts were forgotten.");
        }

        return Ok(());
    }

    config.forget_account(username);
    commit_config(config, config_path)?;
    #[cfg(all(target_os = "linux", feature = "secret-store"))]
    crate::keyring::forget_account(username)?;

    if !quiet {
        println!("The account has been forgotten.");
    }

    Ok(())
}
