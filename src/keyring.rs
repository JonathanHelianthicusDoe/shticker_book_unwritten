#![cfg(all(target_os = "linux", feature = "secret-store"))]

use crate::{config::Config, error::Error};
use secret_service::{blocking::SecretService, EncryptionType};
use std::{collections::HashMap, path::Path};

const APP_ID: &str = "app_id";
const APP_ID_VALUE: &str = "shticker_book_unwritten";
const SECRET_ITEM_LABEL: &str = "Toontown Credentials";
const SECRET_ITEM_ATTRIBUTE: &str = "user";

pub(super) fn get_saved_password(
    _config: &Config,
    username: &str,
) -> Result<Option<String>, Error> {
    let secret_service = SecretService::connect(EncryptionType::Dh)
        .map_err(Error::SessionStoreConnectError)?;

    let collection = secret_service
        .get_default_collection()
        .map_err(Error::SessionStoreConnectError)?;

    collection
        .ensure_unlocked()
        .map_err(Error::PasswordUnlockError)?;

    let mut results = collection
        .search_items(HashMap::from([
            (SECRET_ITEM_ATTRIBUTE, username),
            (APP_ID, APP_ID_VALUE),
        ]))
        .map_err(Error::SessionStoreConnectError)?;

    let Some(item) = results.pop() else {
        return Ok(None);
    };

    item.ensure_unlocked().map_err(Error::PasswordUnlockError)?;

    let secret = item.get_secret().map_err(Error::PasswordGetError)?;

    Ok(Some(
        String::from_utf8(secret).map_err(Error::PasswordUtf8Error)?,
    ))
}

pub(super) fn save_password<P: AsRef<Path>>(
    _config: &mut Config,
    _config_path: P,
    username: String,
    password: String,
) -> Result<(), Error> {
    let secret_service = SecretService::connect(EncryptionType::Dh)
        .map_err(Error::SessionStoreConnectError)?;

    let collection = secret_service
        .get_default_collection()
        .map_err(Error::SessionStoreConnectError)?;

    collection
        .ensure_unlocked()
        .map_err(Error::PasswordUnlockError)?;

    collection
        .create_item(
            SECRET_ITEM_LABEL,
            HashMap::from([
                (SECRET_ITEM_ATTRIBUTE, username.as_str()),
                (APP_ID, APP_ID_VALUE),
            ]),
            password.as_bytes(),
            true, // replace
            "text/plain",
        )
        .map_err(Error::PasswordSaveError)?;

    Ok(())
}

pub(super) fn stored_accounts() -> Result<Vec<String>, Error> {
    let secret_service = SecretService::connect(EncryptionType::Dh)
        .map_err(Error::SessionStoreConnectError)?;

    let collection = secret_service
        .get_default_collection()
        .map_err(Error::SessionStoreConnectError)?;

    collection
        .ensure_unlocked()
        .map_err(Error::PasswordUnlockError)?;

    let results = collection
        .search_items(HashMap::from([(APP_ID, APP_ID_VALUE)]))
        .map_err(Error::SessionStoreConnectError)?;

    results
        .into_iter()
        .map(|item| {
            item.ensure_unlocked().map_err(Error::PasswordUnlockError)?;

            let attributes =
                item.get_attributes().map_err(Error::PasswordGetError)?;

            let username = attributes.get(SECRET_ITEM_ATTRIBUTE).cloned();

            Ok(username)
        })
        .filter_map(|res| res.transpose())
        .collect()
}
