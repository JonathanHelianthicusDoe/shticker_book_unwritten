use reqwest;
use serde_json;
use std::{error, fmt, io, path::PathBuf};

#[derive(Debug)]
pub enum Error {
    NoPossibleConfigPath,
    BadConfigPath(PathBuf),
    MkdirError(PathBuf, io::Error),
    PermissionDenied(String, io::Error),
    StdoutError(io::Error),
    StdinError(io::Error),
    UnknownIoError(String, io::Error),
    SerializeError(serde_json::Error),
    DeserializeError(serde_json::Error),
    ManifestRequestError(reqwest::Error),
    ManifestRequestStatusError(reqwest::StatusCode),
    BadManifestFormat(String),
    FileReadError(PathBuf, io::Error),
    FileWriteError(PathBuf, io::Error),
    DownloadRequestError(reqwest::Error),
    DownloadRequestStatusError(reqwest::StatusCode),
    CopyIntoFileError(PathBuf, reqwest::Error),
    DecodeError(PathBuf, io::Error),
    BadPatchVersion,
    BadPatchSize,
    SeekError(PathBuf, io::Error),
    PatchSanityCheckFail(u8),
    FileRenameError(PathBuf, PathBuf),
    NotDir(PathBuf),
    RemoveFileError(PathBuf, io::Error),
    #[allow(dead_code)]
    MissingFile(&'static str),
    #[allow(dead_code)]
    PermissionsSetError(PathBuf, io::Error),
    MissingCommandLineArg(&'static str),
    PasswordReadError(io::Error),
    HttpClientCreateError(reqwest::Error),
    PostError(reqwest::Error),
    BadLoginResponse(&'static str),
    UnexpectedSuccessValue(String),
    ThreadSpawnError(io::Error),
    ThreadJoinError(io::Error),
    ProcessKillError(u32, io::Error),
    HashMismatch(PathBuf, [u8; 20]),
    InvalidArgValue(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoPossibleConfigPath => {
                #[cfg(target_os = "linux")]
                const MSG: &str = "No config path was given, and the \
                                   $XDG_CONFIG_HOME and $HOME environment \
                                   variables are both unset or empty";
                #[cfg(windows)]
                const MSG: &str = "No config path was given, and the \
                                   %APPDATA% environment variable is unset \
                                   or empty";
                #[cfg(target_os = "macos")]
                const MSG: &str = "No config path was given, and the $HOME \
                                   environment variable is unset or empty";

                f.write_str(MSG)
            }
            Self::BadConfigPath(bcp) => {
                write!(f, "Bad config file path specified: {}", bcp.display())
            }
            Self::MkdirError(path, ioe) => {
                write!(f, "`mkdir -p {:?}` failed:\n\t{}", path, ioe)
            }
            Self::PermissionDenied(info, ioe) => {
                write!(f, "Permission denied while {}:\n\t{}", info, ioe)
            }
            Self::StdoutError(ioe) => write!(f, "stdout error:\n\t{}", ioe),
            Self::StdinError(ioe) => write!(f, "stdin error:\n\t{}", ioe),
            Self::UnknownIoError(info, ioe) => {
                write!(f, "Unknown I/O error while {}:\n\t{}", info, ioe)
            }
            Self::SerializeError(se) => {
                write!(f, "Failed to write JSON:\n\t{}", se)
            }
            Self::DeserializeError(de) => {
                write!(f, "Failed to read JSON:\n\t{}", de)
            }
            Self::ManifestRequestError(mre) => {
                write!(f, "Error requesting manifest:\n\t{}", mre)
            }
            Self::ManifestRequestStatusError(sc) => write!(
                f,
                "Bad status code after requesting manifest:\n\t{}",
                sc,
            ),
            Self::BadManifestFormat(s) => {
                write!(f, "Bad manifest format:\n\t{}", s)
            }
            Self::FileReadError(path, ioe) => {
                write!(f, "Failed to read from {:?}:\n\t{}", path, ioe)
            }
            Self::FileWriteError(path, ioe) => {
                write!(f, "Failed to write to {:?}:\n\t{}", path, ioe)
            }
            Self::DownloadRequestError(dre) => {
                write!(f, "Error requesting download: {}", dre)
            }
            Self::DownloadRequestStatusError(sc) => {
                write!(f, "Bad status code after requesting download: {}", sc)
            }
            Self::CopyIntoFileError(path, cife) => write!(
                f,
                "Failure copying HTTP-downloaded data into {:?}:\n\t{}",
                path, cife,
            ),
            Self::DecodeError(path, ioe) => write!(
                f,
                "Error decoding bzip2 in file {:?}:\n\t{}",
                path, ioe,
            ),
            Self::BadPatchVersion => f.write_str(
                "Unable to determine patch's version, or patch is invalid",
            ),
            Self::BadPatchSize => f.write_str(
                "Unable to determine patch's size, or patch is invalid",
            ),
            Self::SeekError(path, ioe) => write!(
                f,
                "Error while seeking through file {:?}:\n\t{}",
                path, ioe,
            ),
            Self::PatchSanityCheckFail(i) => {
                write!(f, "During patching, sanity check #{} failed", i)
            }
            Self::FileRenameError(from, to) => {
                write!(f, "Error renaming file from {:?} to {:?}", from, to)
            }
            Self::NotDir(path) => write!(f, "{:?} is not a directory", path),
            Self::RemoveFileError(path, ioe) => {
                write!(f, "Error removing file {:?}:\n\t{}", path, ioe)
            }
            Self::MissingFile(name) => {
                write!(f, "Expected \"{}\" file to exist", name)
            }
            Self::PermissionsSetError(path, ioe) => write!(
                f,
                "Failure to set permissions on file {:?}:\n\t{}",
                path, ioe,
            ),
            Self::MissingCommandLineArg(a) => write!(
                f,
                "Expected the {} command line argument to be present",
                a,
            ),
            Self::PasswordReadError(ioe) => {
                write!(f, "Error reading password:\n\t{}", ioe)
            }
            Self::HttpClientCreateError(hcce) => {
                write!(f, "Error creating HTTP client:\n\t{}", hcce)
            }
            Self::PostError(pe) => {
                write!(f, "Error sending HTTP POST:\n\t{}", pe)
            }
            Self::BadLoginResponse(blr) => {
                write!(f, "Bad login response:\n\t{}", blr)
            }
            Self::UnexpectedSuccessValue(value) => {
                write!(f, "Unexpected \"success\" value: {}", value)
            }
            Self::ThreadSpawnError(ioe) => {
                write!(f, "Error spawning thread:\n\t{}", ioe)
            }
            Self::ThreadJoinError(ioe) => {
                write!(f, "Error attempting to join thread:\n\t{}", ioe)
            }
            Self::ProcessKillError(pid, ioe) => write!(
                f,
                "Error killing child process with pid {}:\n\t{}",
                pid, ioe,
            ),
            Self::HashMismatch(path, expected) => {
                write!(
                    f,
                    "SHA1 hash of local file {:?} did not match manifest's \
                     hash of ",
                    path,
                )?;
                for b in expected.iter() {
                    write!(f, "{:02x}", b)?;
                }

                Ok(())
            }
            Self::InvalidArgValue(param) => {
                write!(f, "Invalid value for the argument of {}", param)
            }
        }
    }
}

impl error::Error for Error {}

impl Error {
    pub fn return_code(&self) -> i32 {
        match self {
            Self::NoPossibleConfigPath => 1,
            Self::BadConfigPath(_) => 2,
            Self::MkdirError(_, _) => 3,
            Self::PermissionDenied(_, _) => 4,
            Self::StdoutError(_) => 5,
            Self::StdinError(_) => 6,
            Self::UnknownIoError(_, _) => 7,
            Self::SerializeError(_) => 8,
            Self::DeserializeError(_) => 9,
            Self::ManifestRequestError(_) => 10,
            Self::ManifestRequestStatusError(_) => 11,
            Self::BadManifestFormat(_) => 12,
            Self::FileReadError(_, _) => 13,
            Self::FileWriteError(_, _) => 14,
            Self::DownloadRequestError(_) => 15,
            Self::DownloadRequestStatusError(_) => 16,
            Self::CopyIntoFileError(_, _) => 17,
            Self::DecodeError(_, _) => 18,
            Self::BadPatchVersion => 19,
            Self::BadPatchSize => 20,
            Self::SeekError(_, _) => 21,
            Self::PatchSanityCheckFail(_) => 22,
            Self::FileRenameError(_, _) => 23,
            Self::NotDir(_) => 24,
            Self::RemoveFileError(_, _) => 25,
            Self::MissingFile(_) => 26,
            Self::PermissionsSetError(_, _) => 27,
            Self::MissingCommandLineArg(_) => 28,
            Self::PasswordReadError(_) => 29,
            Self::HttpClientCreateError(_) => 30,
            Self::PostError(_) => 31,
            Self::BadLoginResponse(_) => 32,
            Self::UnexpectedSuccessValue(_) => 33,
            Self::ThreadSpawnError(_) => 34,
            Self::ThreadJoinError(_) => 35,
            Self::ProcessKillError(_, _) => 36,
            Self::HashMismatch(_, _) => 37,
            Self::InvalidArgValue(_) => 38,
        }
    }
}
