use reqwest;
use serde_json;
use std::{error, fmt, io, path::PathBuf};

#[derive(Debug)]
pub enum Error {
    NoPossibleConfigPath,
    BadConfigPath(PathBuf),
    MkdirError(io::Error),
    PermissionDenied(io::Error),
    StdoutError(io::Error),
    StdinError(io::Error),
    UnknownIoError(io::Error),
    SerializeError(serde_json::Error),
    DeserializeError(serde_json::Error),
    ManifestRequestError(reqwest::Error),
    ManifestRequestStatusError(reqwest::StatusCode),
    BadManifestFormat(String),
    FileReadError(io::Error),
    FileWriteError(io::Error),
    DownloadRequestError(reqwest::Error),
    DownloadRequestStatusError(reqwest::StatusCode),
    CopyIntoFileError(reqwest::Error),
    DecodeError(io::Error),
    BadPatchVersion,
    BadPatchSize,
    SeekError(io::Error),
    PatchSanityCheckFail(u8),
    FileRenameError(io::Error),
    NotDir(PathBuf),
    RemoveFileError(io::Error),
    #[allow(dead_code)]
    MissingFile(&'static str),
    #[allow(dead_code)]
    PermissionsSetError(io::Error),
    MissingCommandLineArg(&'static str),
    PasswordReadError(io::Error),
    HttpClientCreateError(reqwest::Error),
    PostError(reqwest::Error),
    BadLoginResponse(&'static str),
    UnexpectedSuccessValue(String),
    ThreadSpawnError(io::Error),
    ThreadJoinError(io::Error),
    ProcessKillError(io::Error),
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
            },
            Self::BadConfigPath(bcp) =>
                write!(f, "Bad config file path specified: {}", bcp.display()),
            Self::MkdirError(ioe) =>
                write!(f, "`mkdir -p` failed:\n\t{}", ioe),
            Self::PermissionDenied(ioe) =>
                write!(f, "Permission denied:\n\t{}", ioe),
            Self::StdoutError(ioe) => write!(f, "stdout error:\n\t{}", ioe),
            Self::StdinError(ioe) => write!(f, "stdin error:\n\t{}", ioe),
            Self::UnknownIoError(ioe) =>
                write!(f, "Unknown I/O error:\n\t{}", ioe),
            Self::SerializeError(se) =>
                write!(f, "Failed to write JSON:\n\t{}", se),
            Self::DeserializeError(de) =>
                write!(f, "Failed to read JSON:\n\t{}", de),
            Self::ManifestRequestError(mre) =>
                write!(f, "Error requesting manifest:\n\t{}", mre),
            Self::ManifestRequestStatusError(sc) => write!(
                f,
                "Bad status code after requesting manifest:\n\t{}",
                sc,
            ),
            Self::BadManifestFormat(s) =>
                write!(f, "Bad manifest format:\n\t{}", s),
            Self::FileReadError(ioe) =>
                write!(f, "Failed to read from file:\n\t{}", ioe),
            Self::FileWriteError(ioe) =>
                write!(f, "Failed to write to file:\n\t{}", ioe),
            Self::DownloadRequestError(dre) =>
                write!(f, "Error requesting download:\n\t{}", dre),
            Self::DownloadRequestStatusError(sc) => write!(
                f,
                "Bad status code after requesting download:\n\t{}",
                sc,
            ),
            Self::CopyIntoFileError(cife) => write!(
                f,
                "Failure copying HTTP-downloaded data into file:\n\t{}",
                cife
            ),
            Self::DecodeError(ioe) =>
                write!(f, "Error decoding bzip2:\n\t{}", ioe),
            Self::BadPatchVersion => f.write_str(
                "Unable to determine patch's version, or patch is invalid",
            ),
            Self::BadPatchSize => f.write_str(
                "Unable to determine patch's size, or patch is invalid",
            ),
            Self::SeekError(ioe) =>
                write!(f, "Error while seeking through file:\n\t{}", ioe),
            Self::PatchSanityCheckFail(i) =>
                write!(f, "During patching, sanity check #{} failed", i),
            Self::FileRenameError(ioe) =>
                write!(f, "Error renaming file:\n\t{}", ioe),
            Self::NotDir(p) => write!(f, "{} is not a directory", p.display()),
            Self::RemoveFileError(ioe) =>
                write!(f, "Error removing file:\n\t{}", ioe),
            Self::MissingFile(n) => write!(f, "Expected {} file to exist", n),
            Self::PermissionsSetError(ioe) =>
                write!(f, "Failure to set permissions on file:\n\t{}", ioe),
            Self::MissingCommandLineArg(a) => write!(
                f,
                "Expected the {} command line argument to be present",
                a,
            ),
            Self::PasswordReadError(ioe) =>
                write!(f, "Error reading password:\n\t{}", ioe),
            Self::HttpClientCreateError(hcce) =>
                write!(f, "Error creating HTTP client:\n\t{}", hcce),
            Self::PostError(pe) =>
                write!(f, "Error sending HTTP POST:\n\t{}", pe),
            Self::BadLoginResponse(blr) =>
                write!(f, "Bad login response:\n\t{}", blr),
            Self::UnexpectedSuccessValue(usv) =>
                write!(f, "Unexpected \"success\" value: {}", usv),
            Self::ThreadSpawnError(ioe) =>
                write!(f, "Error spawning thread:\n\t{}", ioe),
            Self::ThreadJoinError(ioe) =>
                write!(f, "Error attempting to join thread:\n\t{}", ioe),
            Self::ProcessKillError(ioe) =>
                write!(f, "Error killing child process:\n\t{}", ioe),
        }
    }
}

impl error::Error for Error {}

impl Error {
    pub fn return_code(&self) -> i32 {
        match self {
            Self::NoPossibleConfigPath => 1,
            Self::BadConfigPath(_) => 2,
            Self::MkdirError(_) => 3,
            Self::PermissionDenied(_) => 4,
            Self::StdoutError(_) => 5,
            Self::StdinError(_) => 6,
            Self::UnknownIoError(_) => 7,
            Self::SerializeError(_) => 8,
            Self::DeserializeError(_) => 9,
            Self::ManifestRequestError(_) => 10,
            Self::ManifestRequestStatusError(_) => 11,
            Self::BadManifestFormat(_) => 12,
            Self::FileReadError(_) => 13,
            Self::FileWriteError(_) => 14,
            Self::DownloadRequestError(_) => 15,
            Self::DownloadRequestStatusError(_) => 16,
            Self::CopyIntoFileError(_) => 17,
            Self::DecodeError(_) => 18,
            Self::BadPatchVersion => 19,
            Self::BadPatchSize => 20,
            Self::SeekError(_) => 21,
            Self::PatchSanityCheckFail(_) => 22,
            Self::FileRenameError(_) => 23,
            Self::NotDir(_) => 24,
            Self::RemoveFileError(_) => 25,
            Self::MissingFile(_) => 26,
            Self::PermissionsSetError(_) => 27,
            Self::MissingCommandLineArg(_) => 28,
            Self::PasswordReadError(_) => 29,
            Self::HttpClientCreateError(_) => 30,
            Self::PostError(_) => 31,
            Self::BadLoginResponse(_) => 32,
            Self::UnexpectedSuccessValue(_) => 33,
            Self::ThreadSpawnError(_) => 34,
            Self::ThreadJoinError(_) => 35,
            Self::ProcessKillError(_) => 36,
        }
    }
}
