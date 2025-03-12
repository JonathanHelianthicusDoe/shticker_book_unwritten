#[cfg(all(target_os = "linux", feature = "secret-store"))]
use crate::keyring::{get_saved_password, save_password};
use crate::{config::Config, error::Error};
use reqwest::{blocking as rb, header};
use rpassword;
use serde::Serialize;
use std::{
    collections::BTreeMap,
    ffi::OsStr,
    io::{self, Write},
    path::Path,
    process, thread,
    time::{Duration, Instant},
};

const LOGIN_API_URI: &str =
    "https://www.toontownrewritten.com/api/login?format=json";

#[cfg(not(all(target_os = "linux", feature = "secret-store")))]
fn get_saved_password(
    config: &Config,
    username: &str,
) -> Result<Option<String>, Error> {
    Ok(config
        .accounts
        .get(username)
        .and_then(|val| {
            if let serde_json::Value::String(p) = val {
                Some(p)
            } else {
                None
            }
        })
        .cloned())
}

#[cfg(not(all(target_os = "linux", feature = "secret-store")))]
fn save_password<P: AsRef<Path>>(
    config: &mut Config,
    config_path: P,
    username: String,
    password: String,
) -> Result<(), Error> {
    config.add_account(username, password);
    crate::config::commit_config(config, config_path)?;

    Ok(())
}

pub fn login<'a, P: AsRef<Path>, A: Iterator<Item = &'a str>>(
    config: &mut Config,
    config_path: P,
    client: &rb::Client,
    quiet: bool,
    argv: A,
    children: &mut Vec<(String, process::Child, Instant)>,
) -> Result<(), Error> {
    let (mut usernames, mut no_save) = (Vec::new(), false);
    for arg in argv {
        match arg {
            "-n" | "--no-save" => no_save = true,
            _ => usernames.push(arg),
        }
    }

    let mut username_buf = String::new();

    if !usernames.is_empty() {
        for username in usernames {
            if let Some(password) = get_saved_password(config, username)? {
                if !quiet {
                    println!("Using saved password...");
                }

                handle_name_and_pw(
                    config,
                    config_path.as_ref(),
                    client,
                    quiet,
                    no_save,
                    username.to_owned(),
                    password,
                )?
                .map(|c| children.push(c));
            } else {
                print!("Password for {}: ", username);
                io::stdout().flush().map_err(Error::StdoutError)?;

                handle_name_and_pw(
                    config,
                    config_path.as_ref(),
                    client,
                    quiet,
                    no_save,
                    username.to_owned(),
                    rpassword::read_password()
                        .map_err(Error::PasswordReadError)?,
                )?
                .map(|c| children.push(c));
            }
        }
    } else {
        print!("Username: ");
        io::stdout().flush().map_err(Error::StdoutError)?;
        username_buf.reserve(0x10);
        io::stdin()
            .read_line(&mut username_buf)
            .map_err(Error::StdinError)?;
        username_buf.truncate(username_buf.trim_end().len());

        let password = if let Some(password) =
            get_saved_password(config, &username_buf)?
        {
            if !quiet {
                println!("Using saved password...");
            }

            password
        } else {
            print!("Password for {}: ", username_buf);
            io::stdout().flush().map_err(Error::StdoutError)?;

            rpassword::read_password().map_err(Error::PasswordReadError)?
        };

        handle_name_and_pw(
            config,
            config_path,
            client,
            quiet,
            no_save,
            username_buf,
            password,
        )?
        .map(|c| children.push(c));
    }

    Ok(())
}

fn handle_name_and_pw<P: AsRef<Path>>(
    config: &mut Config,
    config_path: P,
    client: &rb::Client,
    quiet: bool,
    no_save: bool,
    username: String,
    password: String,
) -> Result<Option<(String, process::Child, Instant)>, Error> {
    let mut params = BTreeMap::new();
    params.insert("username", username.as_str());
    params.insert("password", password.as_str());
    if let Some(response_json) = handle_login_negotiation(
        client,
        quiet,
        post_to_login_api(client, &params)?,
    )? {
        if !no_save {
            let new_account = get_saved_password(config, &username)?.is_none();
            save_password(config, config_path, username.clone(), password)?;
            if !quiet && new_account {
                println!("New account saved in config!");
            }
        }

        let play_cookie = response_json
            .get("cookie")
            .and_then(|val| {
                if let serde_json::Value::String(c) = val {
                    Some(c)
                } else {
                    None
                }
            })
            .ok_or(Error::BadLoginResponse(
                "Expected \"cookie\" key with String value",
            ))?;
        let game_server = response_json
            .get("gameserver")
            .and_then(|val| {
                if let serde_json::Value::String(gs) = val {
                    Some(gs)
                } else {
                    None
                }
            })
            .ok_or(Error::BadLoginResponse(
                "Expected \"gameserver\" key with String value",
            ))?;

        let ret = launch(config, quiet, play_cookie, game_server)
            .map(|c| Some((username, c, Instant::now())));
        if !quiet && ret.is_ok() {
            println!("Game launched successfully!");
        }

        ret
    } else {
        Ok(None)
    }
}

fn handle_login_negotiation(
    client: &rb::Client,
    quiet: bool,
    mut response_json: serde_json::Value,
) -> Result<Option<serde_json::Value>, Error> {
    loop {
        let success = response_json
            .get("success")
            .and_then(|val| match val {
                serde_json::Value::String(s) => Some(s.as_str()),
                serde_json::Value::Bool(b) => {
                    if *b {
                        Some("true")
                    } else {
                        Some("false")
                    }
                }
                _ => None,
            })
            .ok_or(Error::BadLoginResponse(
                "Expected \"success\" key with a String or Boolean value",
            ))?;

        match success {
            "true" => {
                if !quiet {
                    println!("Authentication success!");
                }

                return Ok(Some(response_json));
            }
            "delayed" => {
                response_json = enqueue(client, quiet, &response_json)?
            }
            "partial" => {
                response_json =
                    if let Some(rj) = do_2fa(client, &response_json)? {
                        rj
                    } else {
                        return Ok(None);
                    }
            }
            "false" => {
                println!(
                    "Login failed: {}",
                    response_json
                        .get("banner")
                        .and_then(|val| {
                            if let serde_json::Value::String(s) = val {
                                Some(s)
                            } else {
                                None
                            }
                        })
                        .ok_or(Error::BadLoginResponse(
                            "Expected \"banner\" key with String value",
                        ))?,
                );

                return Ok(None);
            }
            _ => {
                return Err(Error::UnexpectedSuccessValue(success.to_owned()))
            }
        }
    }
}

/// Return value is `Ok(None)` if cancelled by user.
fn do_2fa(
    client: &rb::Client,
    response_json: &serde_json::Value,
) -> Result<Option<serde_json::Value>, Error> {
    let auth_token = response_json
        .get("responseToken")
        .and_then(|val| {
            if let serde_json::Value::String(s) = val {
                Some(s)
            } else {
                None
            }
        })
        .ok_or(Error::BadLoginResponse(
            "Expected \"responseToken\" key with String value",
        ))?;

    print!(
        "{}\n(...or type \"cancel\" to cancel):\n> ",
        response_json
            .get("banner")
            .and_then(|val| if let serde_json::Value::String(s) = val {
                Some(s)
            } else {
                None
            })
            .ok_or(Error::BadLoginResponse(
                "Expected \"banner\" key with String value",
            ))?,
    );
    io::stdout().flush().map_err(Error::StdoutError)?;
    let mut app_token = String::with_capacity(0x10);
    io::stdin()
        .read_line(&mut app_token)
        .map_err(Error::StdinError)?;
    app_token.truncate(app_token.trim_end().len());

    if app_token == "cancel" {
        Ok(None)
    } else {
        let mut params = BTreeMap::new();
        params.insert("appToken", app_token.as_str());
        params.insert("authToken", auth_token);

        post_to_login_api(client, &params).map(Some)
    }
}

fn enqueue(
    client: &rb::Client,
    quiet: bool,
    response_json: &serde_json::Value,
) -> Result<serde_json::Value, Error> {
    let eta = response_json
        .get("eta")
        .and_then(|val| match val {
            serde_json::Value::String(s) => s.parse().ok(),
            serde_json::Value::Number(n) => n.as_f64(),
            _ => None,
        })
        .ok_or(Error::BadLoginResponse(
            "Expected \"eta\" key with a String or Number value",
        ))?;
    if !quiet {
        println!(
            "Waiting in queue... ETA: {}, position in line: {}",
            eta,
            response_json
                .get("position")
                .and_then(|val| match val {
                    serde_json::Value::String(s) => s.parse().ok(),
                    serde_json::Value::Number(n) => n.as_u64(),
                    _ => None,
                })
                .ok_or(Error::BadLoginResponse(
                    "Expected \"position\" key with a String or unsigned \
                     Number value",
                ))?,
        );
    }

    let queue_token = response_json
        .get("queueToken")
        .and_then(|val| {
            if let serde_json::Value::String(s) = val {
                Some(s)
            } else {
                None
            }
        })
        .ok_or(Error::BadLoginResponse(
            "Expected \"queueToken\" key with a String value",
        ))?;
    // Be a tad less aggressive if the server is overloaded
    let sleep_time = if eta < 0.25 {
        25
    } else if eta < 5.0 {
        500
    } else {
        1_500
    };

    thread::sleep(Duration::from_millis(sleep_time));

    let mut params = BTreeMap::new();
    params.insert("queueToken", queue_token);

    post_to_login_api(client, &params)
}

fn post_to_login_api<K: Ord + Serialize, V: Serialize>(
    client: &rb::Client,
    params: &BTreeMap<K, V>,
) -> Result<serde_json::Value, Error> {
    serde_json::from_str(
        &client
            .post(LOGIN_API_URI)
            .header(header::ACCEPT, "text/plain")
            .form(&params)
            .send()
            .map_err(Error::PostError)?
            .text()
            .map_err(Error::PostError)?,
    )
    .map_err(Error::DeserializeError)
}

fn launch<S: AsRef<OsStr>, T: AsRef<OsStr>>(
    config: &Config,
    quiet: bool,
    play_cookie: S,
    game_server: T,
) -> Result<process::Child, Error> {
    if !quiet {
        println!("Launching the game...");
    }

    #[cfg(target_os = "linux")]
    let command_text = "./TTREngine";
    #[cfg(windows)]
    let command_text = {
        #[cfg(target_arch = "x86")]
        const EXE_NAME: &str = "TTREngine.exe";
        #[cfg(target_arch = "x86_64")]
        const EXE_NAME: &str = "TTREngine64.exe";

        // `.current_dir(&config.install_dir)` doesn't seem to work like it
        // does on Linux, so this is just a (naïve) way of making real sure
        // that we are pointing at the right executable.
        let mut command_buf = config.install_dir.clone();
        command_buf.push(EXE_NAME);

        command_buf
    };
    #[cfg(target_os = "macos")]
    let command_text = {
        // `.current_dir` is also borked on macOS.
        let mut command_buf = config.install_dir.clone();
        command_buf.push("Toontown Rewritten");

        command_buf
    };

    process::Command::new(&command_text)
        .current_dir(&config.install_dir)
        .env("TTR_PLAYCOOKIE", play_cookie)
        .env("TTR_GAMESERVER", game_server)
        .stdin(process::Stdio::null())
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .spawn()
        .map_err(Error::ThreadSpawnError)
}
