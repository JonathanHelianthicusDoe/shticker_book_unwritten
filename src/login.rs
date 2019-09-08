use crate::{
    config::{commit_config, Config},
    error::Error,
};
use reqwest::{self, header};
use rpassword;
use serde::Serialize;
use std::{
    collections::BTreeMap,
    ffi::OsStr,
    io::{self, Write},
    path::Path,
    process,
    thread,
    time::{Duration, Instant},
};

const LOGIN_API_URI: &str =
    "https://www.toontownrewritten.com/api/login?format=json";

pub fn login<'a, P: AsRef<Path>, A: Iterator<Item = &'a str>>(
    config: &mut Config,
    config_path: P,
    client: &reqwest::Client,
    argv: A,
) -> Result<Option<(String, process::Child, Instant)>, Error> {
    let (mut maybe_username, mut nosave) = (None, false);
    for arg in argv {
        match arg {
            "-n" | "--no-save" => nosave = true,
            _ =>
                if maybe_username.is_some() {
                    println!("Unexpected argument: {}", arg);

                    return Ok(None);
                } else {
                    maybe_username = Some(arg);
                },
        }
    }

    let (mut username_buf, mut password_buf) = (String::new(), String::new());

    let (username, password) = if let Some(username) = maybe_username {
        if let Some(password) = config.accounts.get(username).and_then(|val| {
            if let serde_json::Value::String(p) = val {
                println!("Using saved password...");

                Some(p)
            } else {
                None
            }
        }) {
            (username, password.as_str())
        } else {
            print!("Password for {}: ", username);
            io::stdout().flush().map_err(Error::StdoutError)?;
            password_buf = rpassword::read_password_from_tty(None)
                .map_err(Error::PasswordReadError)?;

            (username, password_buf.as_str())
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
            config.accounts.get(&username_buf).and_then(|val| {
                if let serde_json::Value::String(p) = val {
                    println!("Using saved password...");

                    Some(p)
                } else {
                    None
                }
            }) {
            password.as_str()
        } else {
            print!("Password for {}: ", username_buf);
            io::stdout().flush().map_err(Error::StdoutError)?;
            password_buf = rpassword::read_password_from_tty(None)
                .map_err(Error::PasswordReadError)?;

            password_buf.as_str()
        };

        (username_buf.as_str(), password)
    };

    let mut params = BTreeMap::new();
    params.insert("username", username);
    params.insert("password", password);
    if let Some(response_json) =
        handle_login_negotiation(client, post_to_login_api(client, &params)?)?
    {
        let username = if username_buf.is_empty() {
            username.to_owned()
        } else {
            username_buf
        };
        if !nosave {
            let password = if password_buf.is_empty() {
                password.to_owned()
            } else {
                password_buf
            };

            let old_acc = config.add_account(username.clone(), password);
            commit_config(config, config_path)?;
            if old_acc.is_none() {
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

        launch(config, play_cookie, game_server)
            .map(|c| Some((username, c, Instant::now())))
    } else {
        Ok(None)
    }
}

fn handle_login_negotiation(
    client: &reqwest::Client,
    mut response_json: serde_json::Value,
) -> Result<Option<serde_json::Value>, Error> {
    loop {
        let success = response_json
            .get("success")
            .and_then(|val| match val {
                serde_json::Value::String(s) => Some(s.as_str()),
                serde_json::Value::Bool(b) =>
                    if *b {
                        Some("true")
                    } else {
                        Some("false")
                    },
                _ => None,
            })
            .ok_or(Error::BadLoginResponse(
                "Expected \"success\" key with a String or Boolean value",
            ))?;

        match success {
            "true" => {
                println!("Authentication success!");

                return Ok(Some(response_json));
            },
            "delayed" => response_json = enqueue(client, &response_json)?,
            "partial" =>
                response_json =
                    if let Some(rj) = do_2fa(client, &response_json)? {
                        rj
                    } else {
                        return Ok(None);
                    },
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
            },
            _ =>
                return Err(Error::UnexpectedSuccessValue(success.to_owned())),
        }
    }
}

/// Return value is `Ok(None)` if cancelled by user.
fn do_2fa(
    client: &reqwest::Client,
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
    client: &reqwest::Client,
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
                "Expected \"position\" key with a String or unsigned Number \
                 value",
            ))?,
    );

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
    let sleep_time = if eta < 5.0 { 500 } else { 1_500 };

    thread::sleep(Duration::from_millis(sleep_time));

    let mut params = BTreeMap::new();
    params.insert("queueToken", queue_token);

    post_to_login_api(client, &params)
}

fn post_to_login_api<K: Ord + Serialize, V: Serialize>(
    client: &reqwest::Client,
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
    play_cookie: S,
    game_server: T,
) -> Result<process::Child, Error> {
    println!("Launching the game...");

    process::Command::new("./TTREngine")
        .current_dir(&config.install_dir)
        .env("TTR_PLAYCOOKIE", play_cookie)
        .env("TTR_GAMESERVER", game_server)
        .stdin(process::Stdio::null())
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .spawn()
        .map_err(Error::ThreadSpawnError)
}
