
use crate::apperror::AppError;
use std::{io, time};
use std::fmt;

/// An error type containing other error types, useful for convertible Result .
#[derive(Debug)]
pub enum ErrorWrapper {
    Rusqlite(rusqlite::Error),
    Filesystem,
    IO(io::Error),
    SystemTimeError(time::SystemTimeError),
    AppError(AppError),
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

impl From<AppError> for ErrorWrapper {
    fn from(e: AppError) -> Self {
        ErrorWrapper::AppError(e)
    }
}

impl fmt::Display for ErrorWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let buf: String = match self {
            ErrorWrapper::Rusqlite(e) =>        { format!("{}", e) },
            ErrorWrapper::Filesystem =>         { format!("filesystem error.") },
            ErrorWrapper::IO(e) =>              { format!("{}", e) },
            ErrorWrapper::SystemTimeError(e) => { format!("{}", e) },
            ErrorWrapper::AppError(e) =>        { format!("{}", e) },
        };
        write!(f, "ErrorWrapper: {}", buf)
    }
}