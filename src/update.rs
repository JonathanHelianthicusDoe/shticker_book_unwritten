use crate::{config::Config, error::Error};
use bzip2::write::BzDecoder;
use reqwest;
use serde_json;
use sha1::{Digest, Sha1};
use std::{
    fs::File,
    io::{self, prelude::*},
    path::Path,
};

pub const BUFFER_SIZE: usize = 0x20_00;
pub const DEFAULT_ARCH: &str = "linux2";

pub fn update(config: &Config) -> Result<(), Error> {
    let manifest_map = match get_manifest(config)? {
        serde_json::Value::Object(m) => m,
        _ =>
            return Err(Error::BadManifestFormat(
                "Top-level value is not an Object".to_owned(),
            )),
    };

    let mut install_dir = config.install_dir.clone();
    for (i, (file_name, file_obj)) in manifest_map.iter().enumerate() {
        println!(
            "[{:2}/{}] Checking for updates for {}",
            i + 1,
            manifest_map.len(),
            file_name,
        );

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
                    if DEFAULT_ARCH == s {
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
            println!(
                "        Not supported by this OS & architecture, skipping..."
            );

            continue;
        }

        println!("        Checking to see if file already exists...");
        install_dir.push(file_name);
        match File::open(&install_dir) {
            Ok(mut f) => {
                println!("        File exists, checking SHA1 hash...");

                let mut file_buf = [0u8; BUFFER_SIZE];
                let initial_sha = sha_of_reader(&mut f, &mut file_buf)?;

                let manifest_sha =
                    sha_from_hash_str(match file_map.get("hash") {
                        Some(serde_json::Value::String(s)) => s,
                        Some(_) =>
                            return Err(Error::BadManifestFormat(
                                "Value of \"hash\" was not a String"
                                    .to_owned(),
                            )),
                        _ =>
                            return Err(Error::BadManifestFormat(
                                "\"hash\" key missing".to_owned(),
                            )),
                    })?;

                if initial_sha == manifest_sha {
                    println!("        SHA1 hash matches!");

                    continue;
                }

                print!("        SHA1 hash mismatch:\n          Local:    ");
                for b in initial_sha.iter() {
                    print!("{:02x}", b);
                }
                print!("\n          Manifest: ");
                for b in manifest_sha.iter() {
                    print!("{:02x}", b);
                }
                println!("\n        Checking for a patch...");

                let patches_map = file_map
                    .get("patches")
                    .and_then(|val| match val {
                        serde_json::Value::Object(m) => Some(m),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        Error::BadManifestFormat(format!(
                            "Expected \"patches\" key with Object value to \
                             be in the \"{}\" Object",
                            file_name,
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
                                "Expected \"patches\" to be objects"
                                    .to_owned(),
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
                                "Expected \"filename\" key in patch Object"
                                    .to_owned(),
                            )
                        })?;

                    println!("        Found a patch! Downloading it...");

                    let mut extracted_patch_file_name = String::with_capacity(
                        patch_file_name.len() + ".extracted".len(),
                    );
                    extracted_patch_file_name += patch_file_name;
                    extracted_patch_file_name += ".extracted";
                    download_file_to_cache(
                        &mut file_buf,
                        config,
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
                                serde_json::Value::String(s) =>
                                    sha_from_hash_str(s),
                                _ => Err(Error::BadManifestFormat(
                                    "Expected \"compPatchHash\" to be a \
                                     String"
                                        .to_owned(),
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
                                serde_json::Value::String(s) =>
                                    sha_from_hash_str(s),
                                _ => Err(Error::BadManifestFormat(
                                    "Expected \"patchHash\" to be a String"
                                        .to_owned(),
                                )),
                            })?,
                        5,
                    )?;

                    break;
                }
            },
            Err(ioe) => match ioe.kind() {
                io::ErrorKind::NotFound => {
                    println!(
                        "        File doesn't exist, downloading fresh \
                         copy..."
                    );

                    unimplemented!()
                },
                io::ErrorKind::PermissionDenied =>
                    return Err(Error::PermissionDenied(ioe)),
                _ => return Err(Error::UnknownIoError(ioe)),
            },
        }
        install_dir.pop();
    }

    Ok(())
}

fn get_manifest(config: &Config) -> Result<serde_json::Value, Error> {
    let mut manifest_resp = reqwest::get(&config.manifest_uri)
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
) -> Result<[u8; 20], Error> {
    let mut sha = Sha1::default();
    let mut n = buf.len();
    while n == buf.len() {
        n = r.read(buf).map_err(Error::FileReadFailure)?;
        sha.input(&buf[..n]);
    }

    Ok(sha.result().into())
}

fn sha_of_file_by_path<P: AsRef<Path>>(
    path: P,
    buf: &mut [u8],
) -> Result<[u8; 20], Error> {
    let mut file = File::open(path).map_err(|ioe| match ioe.kind() {
        io::ErrorKind::PermissionDenied => Error::PermissionDenied(ioe),
        _ => Error::UnknownIoError(ioe),
    })?;

    let mut sha = Sha1::default();
    let mut n = buf.len();
    while n == buf.len() {
        n = file.read(buf).map_err(Error::FileReadFailure)?;
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
                    "Unexpected character in SHA1 hash string: {}",
                    b as char,
                ))),
        };

        manifest_sha[i / 2] |= nibble_val << if i % 2 == 0 { 4 } else { 0 };
    }

    Ok(manifest_sha)
}

fn download_file_to_cache<S: AsRef<str>>(
    buf: &mut [u8],
    config: &Config,
    file_name: S,
    decompressed_file_name: S,
    compressed_sha: &[u8; 20],
    decompressed_sha: &[u8; 20],
    max_tries: usize,
) -> Result<(), Error> {
    let mut dl_uri =
        String::with_capacity(config.cdn_uri.len() + file_name.as_ref().len());
    dl_uri += &config.cdn_uri;
    dl_uri += file_name.as_ref();

    for i in 1..=max_tries {
        println!(
            "        Downloading {} [attempt {}/{}]",
            file_name.as_ref(),
            i,
            max_tries,
        );

        let mut dl_resp =
            reqwest::get(&dl_uri).map_err(Error::DownloadRequestError)?;
        if !dl_resp.status().is_success() {
            return Err(Error::DownloadRequestStatusError(dl_resp.status()));
        }

        let mut cache_loc = config.cache_dir.clone();
        cache_loc.push(file_name.as_ref());
        {
            let mut dled_file =
                File::create(&cache_loc).map_err(|ioe| match ioe.kind() {
                    io::ErrorKind::PermissionDenied =>
                        Error::PermissionDenied(ioe),
                    _ => Error::UnknownIoError(ioe),
                })?;
            dl_resp
                .copy_to(&mut dled_file)
                .map_err(Error::CopyIntoFileError)?;
        }

        println!("        Checking SHA1 hash of {}", file_name.as_ref());

        let dled_sha = sha_of_file_by_path(&cache_loc, buf)?;
        if &dled_sha != compressed_sha {
            print!("        SHA1 hash mismatch:\n          Local:    ");
            for b in dled_sha.iter() {
                print!("{:02x}", b);
            }
            print!("\n          Manifest: ");
            for b in compressed_sha.iter() {
                print!("{:02x}", b);
            }
            println!("\n        Re-downloading...");

            continue;
        }

        println!("        SHA1 hash matches! Extracting...");

        let compressed_path = cache_loc.clone();
        cache_loc.pop();
        cache_loc.push(decompressed_file_name.as_ref());
        decompress_file(buf, &compressed_path, &cache_loc)?;

        println!("        Checking SHA1 hash of extracted file...");

        let extracted_sha = sha_of_file_by_path(&cache_loc, buf)?;
        if &extracted_sha != decompressed_sha {
            print!("        SHA1 hash mismatch:\n          Local:    ");
            for b in extracted_sha.iter() {
                print!("{:02x}", b);
            }
            print!("\n          Manifest: ");
            for b in decompressed_sha.iter() {
                print!("{:02x}", b);
            }
            println!("\n        Re-downloading...");

            continue;
        }

        println!("        SHA1 hash matches!");

        break;
    }

    Ok(())
}

fn decompress_file<P: AsRef<Path>>(
    buf: &mut [u8],
    compressed_path: P,
    decompress_path: P,
) -> Result<(), Error> {
    let decompressed_file =
        File::create(decompress_path).map_err(|ioe| match ioe.kind() {
            io::ErrorKind::PermissionDenied => Error::PermissionDenied(ioe),
            _ => Error::UnknownIoError(ioe),
        })?;
    let mut decoder = BzDecoder::new(decompressed_file);

    let mut compressed_file =
        File::open(compressed_path).map_err(|ioe| match ioe.kind() {
            io::ErrorKind::PermissionDenied => Error::PermissionDenied(ioe),
            _ => Error::UnknownIoError(ioe),
        })?;

    let mut n = buf.len();
    while n == buf.len() {
        n = compressed_file.read(buf).map_err(Error::FileReadFailure)?;
        decoder
            .write_all(buf.as_ref())
            .map_err(Error::DecodeError)?;
    }

    decoder.finish().map(|_| ()).map_err(Error::DecodeError)
}
