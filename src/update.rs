use crate::{config::Config, error::Error, patch, util};
use bzip2::write::BzDecoder as BzWriteDecoder;
use reqwest::blocking as rb;
use serde_json;
use sha1::{Digest, Sha1};
use std::{
    fs::{self, File},
    io::{self, prelude::*},
    path::{Path, PathBuf},
};

pub const BUFFER_SIZE: usize = 0x20_00;
#[cfg(target_os = "linux")]
pub const OS_AND_ARCH: &str = "linux2";
#[cfg(target_os = "macos")]
pub const OS_AND_ARCH: &str = "darwin";
#[cfg(all(windows, target_arch = "x86_64"))]
pub const OS_AND_ARCH: &str = "win64";
#[cfg(all(windows, target_arch = "x86"))]
pub const OS_AND_ARCH: &str = "win32";

pub fn update(
    config: &Config,
    client: &rb::Client,
    quiet: bool,
) -> Result<(), Error> {
    ensure_dir(&config.install_dir)?;
    ensure_dir(&config.cache_dir)?;

    let manifest_map = match get_manifest(config, client)? {
        serde_json::Value::Object(m) => m,
        _ =>
            return Err(Error::BadManifestFormat(
                "Top-level value is not an Object".to_owned(),
            )),
    };

    let mut install_dir = config.install_dir.clone();
    for (i, (file_name, file_obj)) in manifest_map.iter().enumerate() {
        if !quiet {
            println!(
                "[{:2}/{}] Checking for updates for {}",
                i + 1,
                manifest_map.len(),
                file_name,
            );
        }

        let file_map = if let serde_json::Value::Object(m) = file_obj {
            m
        } else {
            return Err(Error::BadManifestFormat(
                "Expected Object at 2nd-to-top level".to_owned(),
            ));
        };

        let supported_archs = match file_map.get("only").ok_or_else(|| {
            Error::BadManifestFormat("Missing the \"only\" key".to_owned())
        })? {
            serde_json::Value::Array(v) => v,
            _ =>
                return Err(Error::BadManifestFormat(
                    "Expected \"only\"'s value to be an Array".to_owned(),
                )),
        };
        let mut supported_by_this_arch = false;
        for arch_val in supported_archs {
            match arch_val {
                serde_json::Value::String(s) =>
                    if OS_AND_ARCH == s {
                        supported_by_this_arch = true;

                        break;
                    },
                _ =>
                    return Err(Error::BadManifestFormat(
                        "Expected OS & architecture values to be Strings"
                            .to_owned(),
                    )),
            }
        }

        if !supported_by_this_arch {
            if !quiet {
                println!(
                    "        Not supported by this OS & architecture, \
                     skipping..."
                );
            }

            continue;
        }

        if !quiet {
            println!("        Checking to see if file already exists...");
        }

        install_dir.push(file_name);

        let already_existing_file = match File::open(&install_dir) {
            Ok(f) => Some(f),
            Err(ioe) => match ioe.kind() {
                io::ErrorKind::NotFound => {
                    if !quiet {
                        println!(
                            "        File doesn't exist, downloading from \
                             scratch..."
                        );
                    }

                    let mut file_buf = [0u8; BUFFER_SIZE];
                    let compressed_file_name = file_map
                        .get("dl")
                        .ok_or_else(|| {
                            Error::BadManifestFormat(
                                "Expected \"dl\"".to_owned(),
                            )
                        })
                        .and_then(|val| match val {
                            serde_json::Value::String(s) => Ok(s),
                            _ => Err(Error::BadManifestFormat(
                                "Expected \"dl\" to be a String".to_owned(),
                            )),
                        })?;
                    let compressed_sha = file_map
                        .get("compHash")
                        .ok_or_else(|| {
                            Error::BadManifestFormat(
                                "Expected \"compHash\"".to_owned(),
                            )
                        })
                        .and_then(|val| match val {
                            serde_json::Value::String(s) =>
                                sha_from_hash_str(s),
                            _ => Err(Error::BadManifestFormat(
                                "Expected \"compHash\" to be a String"
                                    .to_owned(),
                            )),
                        })?;
                    let decompressed_sha = file_map
                        .get("hash")
                        .ok_or_else(|| {
                            Error::BadManifestFormat(
                                "Expected \"hash\"".to_owned(),
                            )
                        })
                        .and_then(|val| match val {
                            serde_json::Value::String(s) =>
                                sha_from_hash_str(s),
                            _ => Err(Error::BadManifestFormat(
                                "Expected \"hash\" to be a String".to_owned(),
                            )),
                        })?;

                    download_file(
                        false,
                        &mut file_buf,
                        config,
                        client,
                        quiet,
                        compressed_file_name,
                        file_name,
                        &compressed_sha,
                        &decompressed_sha,
                        5,
                    )?;

                    None
                },
                io::ErrorKind::PermissionDenied =>
                    return Err(Error::PermissionDenied(
                        format!("opening {:?}", install_dir),
                        ioe,
                    )),
                _ =>
                    return Err(Error::UnknownIoError(
                        format!("opening {:?}", install_dir),
                        ioe,
                    )),
            },
        };
        if let Some(f) = already_existing_file {
            update_existing_file(
                config,
                client,
                quiet,
                f,
                file_map,
                file_name,
                &install_dir,
            )?;
        }

        install_dir.pop();
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        #[cfg(target_os = "linux")]
        const EXE_NAME: &str = "TTREngine";
        #[cfg(target_os = "macos")]
        const EXE_NAME: &str = "Toontown Rewritten";

        if !quiet {
            println!("Making sure {} is executable...", EXE_NAME);
        }

        install_dir.push(EXE_NAME);
        let mut ttrengine_perms = fs::metadata(&install_dir)
            .map_err(|ioe| match ioe.kind() {
                io::ErrorKind::NotFound => Error::MissingFile(EXE_NAME),
                io::ErrorKind::PermissionDenied => Error::PermissionDenied(
                    format!("obtaining metadata for {:?}", install_dir),
                    ioe,
                ),
                _ => Error::UnknownIoError(
                    format!("obtaining metadata for {:?}", install_dir),
                    ioe,
                ),
            })?
            .permissions();
        let ttrengine_mode = ttrengine_perms.mode();
        if (ttrengine_mode & 0o100) == 0 {
            if !quiet {
                println!(
                    "{} isn't executable, setting executable bit...",
                    EXE_NAME,
                );
            }

            ttrengine_perms.set_mode(ttrengine_mode | 0o700);
            fs::set_permissions(&install_dir, ttrengine_perms)
                .map_err(Error::PermissionsSetError)?;

            if !quiet {
                println!("{} is now executable!", EXE_NAME);
            }
        } else if !quiet {
            println!("{} is already executable!", EXE_NAME);
        }
    }

    Ok(())
}

fn update_existing_file<S: AsRef<str>, P: AsRef<Path>>(
    config: &Config,
    client: &rb::Client,
    quiet: bool,
    mut already_existing_file: File,
    file_map: &serde_json::Map<String, serde_json::Value>,
    file_name: S,
    full_file_path: P,
) -> Result<(), Error> {
    if !quiet {
        println!("        File exists, checking SHA1 hash...");
    }

    let mut file_buf = [0u8; BUFFER_SIZE];
    let initial_sha = sha_of_reader(&mut already_existing_file, &mut file_buf)
        .map_err(|ioe| {
            Error::FileReadError(full_file_path.as_ref().to_path_buf(), ioe)
        })?;

    let manifest_sha = sha_from_hash_str(match file_map.get("hash") {
        Some(serde_json::Value::String(s)) => s,
        Some(_) =>
            return Err(Error::BadManifestFormat(
                "Value of \"hash\" was not a String".to_owned(),
            )),
        _ =>
            return Err(Error::BadManifestFormat(
                "\"hash\" key missing".to_owned(),
            )),
    })?;

    if initial_sha == manifest_sha {
        if !quiet {
            println!("        SHA1 hash matches!");
        }

        return Ok(());
    }

    if !quiet {
        print!("        SHA1 hash mismatch:\n          Local:    ");
        for b in initial_sha.iter() {
            print!("{:02x}", b);
        }
        print!("\n          Manifest: ");
        for b in manifest_sha.iter() {
            print!("{:02x}", b);
        }
        println!("\n        Checking for a patch...");
    }

    let patches_map = file_map
        .get("patches")
        .and_then(|val| match val {
            serde_json::Value::Object(m) => Some(m),
            _ => None,
        })
        .ok_or_else(|| {
            Error::BadManifestFormat(format!(
                "Expected \"patches\" key with Object value to be in the \
                 \"{}\" Object",
                file_name.as_ref(),
            ))
        })?;

    let mut did_patch = false;
    for (manifest_sha_str, patch_obj) in patches_map.iter() {
        if sha_from_hash_str(manifest_sha_str)? != initial_sha {
            continue;
        }

        let patch_map = match patch_obj {
            serde_json::Value::Object(m) => m,
            _ =>
                return Err(Error::BadManifestFormat(
                    "Expected \"patches\" to be objects".to_owned(),
                )),
        };

        let patch_file_name = patch_map
            .get("filename")
            .and_then(|val| match val {
                serde_json::Value::String(s) => Some(s),
                _ => None,
            })
            .ok_or_else(|| {
                Error::BadManifestFormat(
                    "Expected \"filename\" key in patch Object".to_owned(),
                )
            })?;

        if !quiet {
            println!("        Found a patch! Downloading it...");
        }

        let mut extracted_patch_file_name =
            String::with_capacity(patch_file_name.len() + ".extracted".len());
        extracted_patch_file_name += patch_file_name;
        extracted_patch_file_name += ".extracted";
        let extracted_patch_path = download_file(
            true,
            &mut file_buf,
            config,
            client,
            quiet,
            patch_file_name,
            &extracted_patch_file_name,
            &patch_map
                .get("compPatchHash")
                .ok_or_else(|| {
                    Error::BadManifestFormat(
                        "Expected \"compPatchHash\"".to_owned(),
                    )
                })
                .and_then(|val| match val {
                    serde_json::Value::String(s) => sha_from_hash_str(s),
                    _ => Err(Error::BadManifestFormat(
                        "Expected \"compPatchHash\" to be a String".to_owned(),
                    )),
                })?,
            &patch_map
                .get("patchHash")
                .ok_or_else(|| {
                    Error::BadManifestFormat(
                        "Expected \"patchHash\"".to_owned(),
                    )
                })
                .and_then(|val| match val {
                    serde_json::Value::String(s) => sha_from_hash_str(s),
                    _ => Err(Error::BadManifestFormat(
                        "Expected \"patchHash\" to be a String".to_owned(),
                    )),
                })?,
            5,
        )?;

        if !quiet {
            println!("        Applying patch...");
        }

        patch::patch_file(&extracted_patch_path, full_file_path)?;

        if !quiet {
            println!("        File patched successfully!");
        }

        did_patch = true;

        break;
    }

    if !did_patch {
        if !quiet {
            println!("        No patches found, downloading from scratch...");
        }

        let compressed_file_name = file_map
            .get("dl")
            .ok_or_else(|| {
                Error::BadManifestFormat("Expected \"dl\"".to_owned())
            })
            .and_then(|val| match val {
                serde_json::Value::String(s) => Ok(s),
                _ => Err(Error::BadManifestFormat(
                    "Expected \"dl\" to be a String".to_owned(),
                )),
            })?;
        let compressed_sha = file_map
            .get("compHash")
            .ok_or_else(|| {
                Error::BadManifestFormat("Expected \"compHash\"".to_owned())
            })
            .and_then(|val| match val {
                serde_json::Value::String(s) => sha_from_hash_str(s),
                _ => Err(Error::BadManifestFormat(
                    "Expected \"compHash\" to be a String".to_owned(),
                )),
            })?;

        download_file(
            false,
            &mut file_buf,
            config,
            client,
            quiet,
            compressed_file_name,
            file_name,
            &compressed_sha,
            &manifest_sha,
            5,
        )?;
    }

    Ok(())
}

fn get_manifest(
    config: &Config,
    client: &rb::Client,
) -> Result<serde_json::Value, Error> {
    let manifest_resp = client
        .get(&config.manifest_uri)
        .send()
        .map_err(Error::ManifestRequestError)?;
    if !manifest_resp.status().is_success() {
        return Err(Error::ManifestRequestStatusError(manifest_resp.status()));
    }

    let manifest_text =
        manifest_resp.text().map_err(Error::ManifestRequestError)?;

    serde_json::from_str(&manifest_text).map_err(Error::DeserializeError)
}

fn sha_of_reader<R: Read>(
    r: &mut R,
    buf: &mut [u8],
) -> Result<[u8; 20], io::Error> {
    let mut sha = Sha1::default();
    let mut n = buf.len();
    while n == buf.len() {
        n = r.read(buf)?;
        sha.input(&buf[..n]);
    }

    Ok(sha.result().into())
}

fn sha_of_file_by_path<P: AsRef<Path>>(
    path: P,
    buf: &mut [u8],
) -> Result<[u8; 20], Error> {
    let mut file = util::open_file(&path)?;

    let mut sha = Sha1::default();
    let mut n = buf.len();
    while n == buf.len() {
        n = file.read(buf).map_err(|ioe| {
            Error::FileReadError(path.as_ref().to_path_buf(), ioe)
        })?;
        sha.input(&buf[..n]);
    }

    Ok(sha.result().into())
}

fn sha_from_hash_str<S: AsRef<str>>(hash_str: S) -> Result<[u8; 20], Error> {
    let mut manifest_sha = [0u8; 20];
    for (i, &b) in hash_str.as_ref().as_bytes().iter().enumerate() {
        let nibble_val = match b {
            b if b >= b'0' && b <= b'9' => b - b'0',
            b'a' | b'A' => 0x0a,
            b'b' | b'B' => 0x0b,
            b'c' | b'C' => 0x0c,
            b'd' | b'D' => 0x0d,
            b'e' | b'E' => 0x0e,
            b'f' | b'F' => 0x0f,
            _ =>
                return Err(Error::BadManifestFormat(format!(
                    "Unexpected character in SHA1 hash string: {:?}",
                    b as char,
                ))),
        };

        manifest_sha[i / 2] |= nibble_val << if i % 2 == 0 { 4 } else { 0 };
    }

    Ok(manifest_sha)
}

/// Downloads to the cache if `to_cache`, otherwise downloads to the main
/// installation directory. Returns the full path to the downloaded file on
/// success.
#[allow(clippy::too_many_arguments)]
fn download_file<S: AsRef<str>, T: AsRef<str>>(
    to_cache: bool,
    buf: &mut [u8],
    config: &Config,
    client: &rb::Client,
    quiet: bool,
    compressed_file_name: S,
    decompressed_file_name: T,
    compressed_sha: &[u8; 20],
    decompressed_sha: &[u8; 20],
    max_tries: usize,
) -> Result<PathBuf, Error> {
    let mut dl_uri = String::with_capacity(
        config.cdn_uri.len() + compressed_file_name.as_ref().len(),
    );
    dl_uri += &config.cdn_uri;
    dl_uri += compressed_file_name.as_ref();

    let compressed_file_path = {
        let mut pb = if to_cache {
            config.cache_dir.clone()
        } else {
            config.install_dir.clone()
        };
        pb.push(compressed_file_name.as_ref());

        pb
    };
    let decompressed_file_path = {
        let mut pb = if to_cache {
            config.cache_dir.clone()
        } else {
            config.install_dir.clone()
        };
        pb.push(decompressed_file_name.as_ref());

        pb
    };

    for i in 1..=max_tries {
        if !quiet {
            println!(
                "        Downloading {} [attempt {}/{}]",
                compressed_file_name.as_ref(),
                i,
                max_tries,
            );
        }

        let mut dl_resp = client
            .get(&dl_uri)
            .send()
            .map_err(Error::DownloadRequestError)?;
        if !dl_resp.status().is_success() {
            return Err(Error::DownloadRequestStatusError(dl_resp.status()));
        }

        {
            let mut dled_file = util::create_file(&compressed_file_path)?;
            dl_resp
                .copy_to(&mut dled_file)
                .map_err(Error::CopyIntoFileError)?;
        }

        if !quiet {
            println!(
                "        Checking SHA1 hash of {}",
                compressed_file_name.as_ref(),
            );
        }

        let dled_sha = sha_of_file_by_path(&compressed_file_path, buf)?;
        if &dled_sha != compressed_sha {
            if !quiet {
                print!("        SHA1 hash mismatch:\n          Local:    ");
                for b in dled_sha.iter() {
                    print!("{:02x}", b);
                }
                print!("\n          Manifest: ");
                for b in compressed_sha.iter() {
                    print!("{:02x}", b);
                }
                println!("\n        Re-downloading...");
            }

            continue;
        }

        if !quiet {
            println!("        SHA1 hash matches! Extracting...");
        }

        decompress_file(buf, &compressed_file_path, &decompressed_file_path)?;

        if !quiet {
            println!("        Checking SHA1 hash of extracted file...");
        }

        let extracted_sha = sha_of_file_by_path(&decompressed_file_path, buf)?;
        if &extracted_sha != decompressed_sha {
            if !quiet {
                print!("        SHA1 hash mismatch:\n          Local:    ");
                for b in extracted_sha.iter() {
                    print!("{:02x}", b);
                }
                print!("\n          Manifest: ");
                for b in decompressed_sha.iter() {
                    print!("{:02x}", b);
                }
                println!("\n        Re-downloading...");
            }

            continue;
        }

        if !quiet {
            println!("        SHA1 hash matches!");
        }

        break;
    }

    if !quiet {
        println!("        Deleting compressed version...");
    }

    fs::remove_file(&compressed_file_path).map_err(Error::RemoveFileError)?;

    if !quiet {
        println!(
            "        {} all done downloading!",
            decompressed_file_name.as_ref(),
        );
    }

    Ok(decompressed_file_path)
}

fn decompress_file<P: AsRef<Path>>(
    buf: &mut [u8],
    compressed_path: P,
    decompress_path: P,
) -> Result<(), Error> {
    let decompressed_file = util::create_file(decompress_path)?;
    let mut decoder = BzWriteDecoder::new(decompressed_file);

    let mut compressed_file = util::open_file(&compressed_path)?;

    let mut n = buf.len();
    while n == buf.len() {
        n = compressed_file.read(buf).map_err(|ioe| {
            Error::FileReadError(compressed_path.as_ref().to_path_buf(), ioe)
        })?;
        decoder.write_all(&buf[..n]).map_err(Error::DecodeError)?;
    }

    decoder.finish().map(|_| ()).map_err(Error::DecodeError)
}

fn ensure_dir<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    match fs::metadata(&path) {
        Ok(md) =>
            if md.is_dir() {
                Ok(())
            } else {
                Err(Error::NotDir(path.as_ref().to_path_buf()))
            },
        Err(ioe) => match ioe.kind() {
            io::ErrorKind::NotFound =>
                fs::create_dir_all(&path).map_err(|ioe| {
                    Error::MkdirError(path.as_ref().to_path_buf(), ioe)
                }),
            io::ErrorKind::PermissionDenied => Err(Error::PermissionDenied(
                format!("obtaining metadata for {:?}", path.as_ref()),
                ioe,
            )),
            _ => Err(Error::UnknownIoError(
                format!("obtaining metadata for {:?}", path.as_ref()),
                ioe,
            )),
        },
    }
}
