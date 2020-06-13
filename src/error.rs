
use std::{io, time};

/// An error type containing other error types, useful for convertible Result .
#[derive(Debug)]
pub enum ErrorWrapper {
    Rusqlite(rusqlite::Error),
    Filesystem,
    IO(io::Error),
    NoneError,
    SystemTimeError(time::SystemTimeError),
    WithMessage(&'static str),
}

impl From<rusqlite::Error> for ErrorWrapper {
    fn from(e: rusqlite::Error) -> ErrorWrapper {
        ErrorWrapper::Rusqlite(e)
    }
}

impl From<(rusqlite::Connection, rusqlite::Error)> for ErrorWrapper {
    fn from(e: (rusqlite::Connection, rusqlite::Error)) -> ErrorWrapper {
        let (_, error) = e;
        ErrorWrapper::Rusqlite(error)
    }
}

impl From<io::Error> for ErrorWrapper {
    fn from(e: io::Error) -> ErrorWrapper {
        ErrorWrapper::IO(e)
    }
}

impl From<time::SystemTimeError> for ErrorWrapper {
    fn from(e: time::SystemTimeError) -> ErrorWrapper {
        ErrorWrapper::SystemTimeError(e)
    }
}

impl From<&'static str> for ErrorWrapper {
    fn from(msg: &'static str) -> Self {
        ErrorWrapper::WithMessage(msg)
    }
}
