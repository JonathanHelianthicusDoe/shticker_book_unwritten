#![cfg(all(target_os = "linux", feature = "secret-store"))]

use crate::{config::Config, error::Error};
use secret_service::{EncryptionType, blocking::SecretService};
use std::{collections::HashMap, path::Path};

const APP_ID: &str = "app_id";
const APP_ID_VALUE: &str = "shticker_book_unwritten";
const SECRET_ITEM_LABEL: &str = "Toontown Credentials";
const SECRET_ITEM_ATTRIBUTE: &str = "user";

pub(super) fn get_saved_password(
    config: &Config,
    username: &str,
) -> Result<Option<String>, Error> {
    if !config.store_passwords {
        return Ok(None);
    }

    let secret_service = SecretService::connect(EncryptionType::Dh)
        .map_err(Error::SessionStoreConnect)?;

    let collection = secret_service
        .get_default_collection()
        .map_err(Error::SessionStoreConnect)?;

    collection
        .ensure_unlocked()
        .map_err(Error::PasswordUnlock)?;

    let mut results = collection
        .search_items(HashMap::from([
            (SECRET_ITEM_ATTRIBUTE, username),
            (APP_ID, APP_ID_VALUE),
        ]))
        .map_err(Error::SessionStoreConnect)?;

    let Some(item) = results.pop() else {
        return Ok(None);
    };

    item.ensure_unlocked().map_err(Error::PasswordUnlock)?;

    let secret = item.get_secret().map_err(Error::PasswordGet)?;

    Ok(Some(
        String::from_utf8(secret).map_err(Error::PasswordUtf8)?,
    ))
}

pub(super) fn save_password<P: AsRef<Path>>(
    config: &mut Config,
    _config_path: P,
    username: String,
    password: String,
) -> Result<(), Error> {
    if !config.store_passwords {
        return Ok(());
    }

    let secret_service = SecretService::connect(EncryptionType::Dh)
        .map_err(Error::SessionStoreConnect)?;

    let collection = secret_service
        .get_default_collection()
        .map_err(Error::SessionStoreConnect)?;

    collection
        .ensure_unlocked()
        .map_err(Error::PasswordUnlock)?;

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
        .map_err(Error::PasswordSave)?;

    Ok(())
}

pub(super) fn stored_accounts() -> Result<Vec<String>, Error> {
    let secret_service = SecretService::connect(EncryptionType::Dh)
        .map_err(Error::SessionStoreConnect)?;

    let collection = secret_service
        .get_default_collection()
        .map_err(Error::SessionStoreConnect)?;

    collection
        .ensure_unlocked()
        .map_err(Error::PasswordUnlock)?;

    let results = collection
        .search_items(HashMap::from([(APP_ID, APP_ID_VALUE)]))
        .map_err(Error::SessionStoreConnect)?;

    results
        .into_iter()
        .map(|item| {
            item.ensure_unlocked().map_err(Error::PasswordUnlock)?;

            let attributes =
                item.get_attributes().map_err(Error::PasswordGet)?;

            let username = attributes.get(SECRET_ITEM_ATTRIBUTE).cloned();

            Ok(username)
        })
        .filter_map(|res| res.transpose())
        .collect()
}

pub(super) fn account_exists(username: &str) -> Result<bool, Error> {
    let secret_service = SecretService::connect(EncryptionType::Dh)
        .map_err(Error::SessionStoreConnect)?;

    let collection = secret_service
        .get_default_collection()
        .map_err(Error::SessionStoreConnect)?;

    collection
        .ensure_unlocked()
        .map_err(Error::PasswordUnlock)?;

    let results = collection
        .search_items(HashMap::from([
            (SECRET_ITEM_ATTRIBUTE, username),
            (APP_ID, APP_ID_VALUE),
        ]))
        .map_err(Error::SessionStoreConnect)?;

    Ok(!results.is_empty())
}

pub(super) fn forget_account(username: &str) -> Result<(), Error> {
    let secret_service = SecretService::connect(EncryptionType::Dh)
        .map_err(Error::SessionStoreConnect)?;

    let collection = secret_service
        .get_default_collection()
        .map_err(Error::SessionStoreConnect)?;

    collection
        .ensure_unlocked()
        .map_err(Error::PasswordUnlock)?;

    let mut results = collection
        .search_items(HashMap::from([
            (SECRET_ITEM_ATTRIBUTE, username),
            (APP_ID, APP_ID_VALUE),
        ]))
        .map_err(Error::SessionStoreConnect)?;

    let Some(item) = results.pop() else {
        return Ok(());
    };

    item.ensure_unlocked().map_err(Error::PasswordUnlock)?;
    item.delete().map_err(Error::DeleteSecretItem)?;

    Ok(())
}
