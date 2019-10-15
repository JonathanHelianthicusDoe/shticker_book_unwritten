use crate::error::Error;
use std::{fs::File, io, path::Path};

pub fn open_file<P: AsRef<Path>>(path: P) -> Result<File, Error> {
    File::open(&path).map_err(|ioe| match ioe.kind() {
        io::ErrorKind::PermissionDenied => Error::PermissionDenied(
            format!("opening {:?}", path.as_ref()),
            ioe,
        ),
        _ =>
            Error::UnknownIoError(format!("opening {:?}", path.as_ref()), ioe),
    })
}

pub fn create_file<P: AsRef<Path>>(path: P) -> Result<File, Error> {
    File::create(&path).map_err(|ioe| match ioe.kind() {
        io::ErrorKind::PermissionDenied => Error::PermissionDenied(
            format!("creating {:?}", path.as_ref()),
            ioe,
        ),
        _ => Error::UnknownIoError(
            format!("creating {:?}", path.as_ref()),
            ioe,
        ),
    })
}
