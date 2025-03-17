//! `accounts`/`logins` command & its subcommands.

use crate::{
    config::{Config, commit_config},
    error::Error,
};
use std::{
    io::{self, Write},
    path::Path,
    process, time,
};

const ACCOUNTS_HELP_TEXT: &str = "\
Account-management subcommands
==============================
accounts forget     Forget the specified account, erasing its username &
  [username]          password from the config and from the Secret Service
                      keyring, where applicable.
accounts savepws    Set the value of store_passwords in your config. Note that
  <true | false>      setting the value to false will NOT cause any passwords
                      to be forgotten.
";

pub(crate) fn accounts_help() {
    print!("{ACCOUNTS_HELP_TEXT}");
}

pub(crate) fn display_accounts(
    config: &Config,
    children: &[(String, process::Child, time::Instant)],
) -> Result<(), Error> {
    #[cfg(not(all(target_os = "linux", feature = "secret-store")))]
    let stored_accounts: Vec<String> = Vec::new();
    #[cfg(all(target_os = "linux", feature = "secret-store"))]
    let stored_accounts = crate::keyring::stored_accounts()?;

    let max_name_len = if let Some(l) = config
        .accounts
        .iter()
        .map(|(un, _)| un.len())
        .chain(stored_accounts.iter().map(std::string::String::len))
        .max()
    {
        l
    } else {
        return Ok(());
    };

    #[cfg(not(all(target_os = "linux", feature = "secret-store")))]
    let accounts = config.accounts.iter().map(|(un, p)| (un, p.is_string()));
    #[cfg(all(target_os = "linux", feature = "secret-store"))]
    let accounts = stored_accounts
        .iter()
        .map(|u| (u, true))
        .chain(config.accounts.iter().map(|(un, p)| (un, p.is_string())));

    for (username, saved_password) in accounts {
        print!(
            "{} {username}   ",
            if children.iter().any(|(un, _, _)| un == username) {
                '*'
            } else {
                ' '
            },
        );
        for _ in 0..max_name_len - username.len() {
            print!(" ");
        }
        println!("Password?: {}", if saved_password { "yes" } else { "no" });
    }

    Ok(())
}

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
        print!("Enter the username of the account to forget: ");
        io::stdout().flush().map_err(Error::Stdout)?;
        io::stdin()
            .read_line(&mut username_buf)
            .map_err(Error::Stdin)?;

        username_buf.trim()
    };

    if !account_exists(config, username)? {
        println!("No account with that username was found.");

        return Ok(());
    }

    let mut confirm_buf = String::with_capacity(4);
    #[cfg(not(all(target_os = "linux", feature = "secret-store")))]
    print!(
        "Are you absolutely certain that you want to forget the account with \
         username {username}?\nThis will ERASE the username & its associated \
         password from your config file.\n\
         Type yes to confirm: "
    );
    #[cfg(all(target_os = "linux", feature = "secret-store"))]
    print!(
        "Are you absolutely certain that you want to forget the account with \
         username {username}?\nThis will ERASE the username & its associated \
         password from both your config file & your Secret Service keyring.\n\
         Type yes to confirm: "
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

pub(crate) fn set_store_passwords<P: AsRef<Path>>(
    config: &mut Config,
    config_path: P,
    quiet: bool,
    val: Option<&str>,
) -> Result<(), Error> {
    let store_passwords = match val {
        Some("true") => true,
        Some("false") => false,
        _ => {
            println!(
                "store_passwords can only have a value of either true or false"
            );

            return Ok(());
        }
    };

    config.store_passwords = store_passwords;
    commit_config(config, &config_path)?;

    if !quiet {
        if store_passwords {
            #[cfg(not(all(target_os = "linux", feature = "secret-store")))]
            println!(
                "Passwords will now be stored IN PLAIN TEXT at\n{}",
                config_path.as_ref().display()
            );
            #[cfg(all(target_os = "linux", feature = "secret-store"))]
            println!(
                "Passwords will now be stored in your default Secret Service \
                 keyring."
            );
        } else {
            println!("Passwords will be neither stored nor read.");
        }
    }

    Ok(())
}
