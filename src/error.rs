use std::{error, fmt, io, path::PathBuf};

#[derive(Debug)]
pub enum Error {
    NoPossibleConfigPath,
    BadConfigPath(PathBuf),
    Mkdir(PathBuf, io::Error),
    PermissionDenied(String, io::Error),
    Stdout(io::Error),
    Stdin(io::Error),
    UnknownIo(String, io::Error),
    Serialize(serde_json::Error),
    Deserialize(serde_json::Error),
    ManifestRequest(reqwest::Error),
    ManifestRequestStatus(reqwest::StatusCode),
    BadManifestFormat(String),
    FileRead(PathBuf, io::Error),
    FileWrite(PathBuf, io::Error),
    DownloadRequest(reqwest::Error),
    DownloadRequestStatus(reqwest::StatusCode),
    CopyIntoFile(PathBuf, reqwest::Error),
    Decode(PathBuf, io::Error),
    BadPatchVersion,
    BadPatchSize,
    Seek(PathBuf, io::Error),
    PatchSanityCheckFail(u8),
    FileRename(PathBuf, PathBuf),
    NotDir(PathBuf),
    RemoveFile(PathBuf, io::Error),
    #[allow(dead_code)]
    MissingFile(&'static str),
    #[allow(dead_code)]
    PermissionsSet(PathBuf, io::Error),
    MissingCommandLineArg(&'static str),
    PasswordRead(io::Error),
    HttpClientCreate(reqwest::Error),
    Post(reqwest::Error),
    BadLoginResponse(&'static str),
    UnexpectedSuccessValue(String),
    ThreadSpawn(io::Error),
    ThreadJoin(io::Error),
    ProcessKill(u32, io::Error),
    HashMismatch(PathBuf, [u8; 20]),
    #[cfg(all(target_os = "linux", feature = "secret-store"))]
    SessionStoreConnect(secret_service::Error),
    #[cfg(all(target_os = "linux", feature = "secret-store"))]
    PasswordUnlock(secret_service::Error),
    #[cfg(all(target_os = "linux", feature = "secret-store"))]
    PasswordGet(secret_service::Error),
    #[cfg(all(target_os = "linux", feature = "secret-store"))]
    PasswordUtf8(std::string::FromUtf8Error),
    #[cfg(all(target_os = "linux", feature = "secret-store"))]
    PasswordSave(secret_service::Error),
    #[cfg(all(target_os = "linux", feature = "secret-store"))]
    DeleteSecretItem(secret_service::Error),
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
            Self::Mkdir(path, ioe) => {
                write!(f, "`mkdir -p {path:?}` failed:\n\t{ioe}")
            }
            Self::PermissionDenied(info, ioe) => {
                write!(f, "Permission denied while {info}:\n\t{ioe}")
            }
            Self::Stdout(ioe) => write!(f, "stdout error:\n\t{ioe}"),
            Self::Stdin(ioe) => write!(f, "stdin error:\n\t{ioe}"),
            Self::UnknownIo(info, ioe) => {
                write!(f, "Unknown I/O error while {info}:\n\t{ioe}")
            }
            Self::Serialize(se) => {
                write!(f, "Failed to write JSON:\n\t{se}")
            }
            Self::Deserialize(de) => {
                write!(f, "Failed to read JSON:\n\t{de}")
            }
            Self::ManifestRequest(mre) => {
                write!(f, "Error requesting manifest:\n\t{mre}")
            }
            Self::ManifestRequestStatus(sc) => {
                write!(f, "Bad status code after requesting manifest:\n\t{sc}",)
            }
            Self::BadManifestFormat(s) => {
                write!(f, "Bad manifest format:\n\t{s}")
            }
            Self::FileRead(path, ioe) => {
                write!(f, "Failed to read from {path:?}:\n\t{ioe}")
            }
            Self::FileWrite(path, ioe) => {
                write!(f, "Failed to write to {path:?}:\n\t{ioe}")
            }
            Self::DownloadRequest(dre) => {
                write!(f, "Error requesting download: {dre}")
            }
            Self::DownloadRequestStatus(sc) => {
                write!(f, "Bad status code after requesting download: {sc}")
            }
            Self::CopyIntoFile(path, cife) => write!(
                f,
                "Failure copying HTTP-downloaded data into {path:?}:\n\t{cife}",
            ),
            Self::Decode(path, ioe) => {
                write!(f, "Error decoding bzip2 in file {path:?}:\n\t{ioe}",)
            }
            Self::BadPatchVersion => f.write_str(
                "Unable to determine patch's version, or patch is invalid",
            ),
            Self::BadPatchSize => f.write_str(
                "Unable to determine patch's size, or patch is invalid",
            ),
            Self::Seek(path, ioe) => write!(
                f,
                "Error while seeking through file {path:?}:\n\t{ioe}",
            ),
            Self::PatchSanityCheckFail(i) => {
                write!(f, "During patching, sanity check #{i} failed")
            }
            Self::FileRename(from, to) => {
                write!(f, "Error renaming file from {from:?} to {to:?}")
            }
            Self::NotDir(path) => write!(f, "{path:?} is not a directory"),
            Self::RemoveFile(path, ioe) => {
                write!(f, "Error removing file {path:?}:\n\t{ioe}")
            }
            Self::MissingFile(name) => {
                write!(f, "Expected \"{name}\" file to exist")
            }
            Self::PermissionsSet(path, ioe) => write!(
                f,
                "Failure to set permissions on file {path:?}:\n\t{ioe}",
            ),
            Self::MissingCommandLineArg(a) => write!(
                f,
                "Expected the {a} command line argument to be present",
            ),
            Self::PasswordRead(ioe) => {
                write!(f, "Error reading password:\n\t{ioe}")
            }
            Self::HttpClientCreate(hcce) => {
                write!(f, "Error creating HTTP client:\n\t{hcce}")
            }
            Self::Post(pe) => {
                write!(f, "Error sending HTTP POST:\n\t{pe}")
            }
            Self::BadLoginResponse(blr) => {
                write!(f, "Bad login response:\n\t{blr}")
            }
            Self::UnexpectedSuccessValue(value) => {
                write!(f, "Unexpected \"success\" value: {value}")
            }
            Self::ThreadSpawn(ioe) => {
                write!(f, "Error spawning thread:\n\t{ioe}")
            }
            Self::ThreadJoin(ioe) => {
                write!(f, "Error attempting to join thread:\n\t{ioe}")
            }
            Self::ProcessKill(pid, ioe) => write!(
                f,
                "Error killing child process with pid {pid}:\n\t{ioe}",
            ),
            Self::HashMismatch(path, expected) => {
                write!(
                    f,
                    "SHA-1 hash of local file {path:?} did not match manifest's \
                     hash of ",
                )?;
                for b in expected {
                    write!(f, "{b:02x}")?;
                }

                Ok(())
            }
            #[cfg(all(target_os = "linux", feature = "secret-store"))]
            Self::SessionStoreConnect(error) => {
                write!(
                    f,
                    "Failed to connect to session password store:\n\t{}",
                    error
                )
            }
            #[cfg(all(target_os = "linux", feature = "secret-store"))]
            Self::PasswordUnlock(error) => {
                write!(
                    f,
                    "Could not unlock password from session store:\n\t{}",
                    error
                )
            }
            #[cfg(all(target_os = "linux", feature = "secret-store"))]
            Self::PasswordGet(error) => {
                write!(
                    f,
                    "Failed to get password from secret store:\n\t{}",
                    error
                )
            }
            #[cfg(all(target_os = "linux", feature = "secret-store"))]
            Self::PasswordUtf8(error) => {
                write!(
                    f,
                    "Password from session store is invalid:\n\t{}",
                    error
                )
            }
            #[cfg(all(target_os = "linux", feature = "secret-store"))]
            Self::PasswordSave(error) => {
                write!(
                    f,
                    "Failed to save password in session store:\n\t{}",
                    error
                )
            }
            #[cfg(all(target_os = "linux", feature = "secret-store"))]
            Self::DeleteSecretItem(error) => write!(
                f,
                "Failed to delete item in secret store:\n\t{}",
                error
            ),
        }
    }
}

impl error::Error for Error {}

impl Error {
    pub fn return_code(&self) -> i32 {
        match self {
            Self::NoPossibleConfigPath => 1,
            Self::BadConfigPath(_) => 2,
            Self::Mkdir(_, _) => 3,
            Self::PermissionDenied(_, _) => 4,
            Self::Stdout(_) => 5,
            Self::Stdin(_) => 6,
            Self::UnknownIo(_, _) => 7,
            Self::Serialize(_) => 8,
            Self::Deserialize(_) => 9,
            Self::ManifestRequest(_) => 10,
            Self::ManifestRequestStatus(_) => 11,
            Self::BadManifestFormat(_) => 12,
            Self::FileRead(_, _) => 13,
            Self::FileWrite(_, _) => 14,
            Self::DownloadRequest(_) => 15,
            Self::DownloadRequestStatus(_) => 16,
            Self::CopyIntoFile(_, _) => 17,
            Self::Decode(_, _) => 18,
            Self::BadPatchVersion => 19,
            Self::BadPatchSize => 20,
            Self::Seek(_, _) => 21,
            Self::PatchSanityCheckFail(_) => 22,
            Self::FileRename(_, _) => 23,
            Self::NotDir(_) => 24,
            Self::RemoveFile(_, _) => 25,
            Self::MissingFile(_) => 26,
            Self::PermissionsSet(_, _) => 27,
            Self::MissingCommandLineArg(_) => 28,
            Self::PasswordRead(_) => 29,
            Self::HttpClientCreate(_) => 30,
            Self::Post(_) => 31,
            Self::BadLoginResponse(_) => 32,
            Self::UnexpectedSuccessValue(_) => 33,
            Self::ThreadSpawn(_) => 34,
            Self::ThreadJoin(_) => 35,
            Self::ProcessKill(_, _) => 36,
            Self::HashMismatch(_, _) => 37,
            #[cfg(all(target_os = "linux", feature = "secret-store"))]
            Self::SessionStoreConnect(_) => 38,
            #[cfg(all(target_os = "linux", feature = "secret-store"))]
            Self::PasswordUnlock(_) => 39,
            #[cfg(all(target_os = "linux", feature = "secret-store"))]
            Self::PasswordGet(_) => 40,
            #[cfg(all(target_os = "linux", feature = "secret-store"))]
            Self::PasswordUtf8(_) => 41,
            #[cfg(all(target_os = "linux", feature = "secret-store"))]
            Self::PasswordSave(_) => 42,
            #[cfg(all(target_os = "linux", feature = "secret-store"))]
            Self::DeleteSecretItem(_) => 43,
        }
    }
}
