
use std::{io, time};

/// An error type for all in-house funcs & methods.
#[derive(Debug)]
pub enum AppError {
    /// For use when an `Option` was `None`, but `Some(a)` was expected.
    /// Workaround until the `Try` trait becomes stable.
    NoneError,

    WithMessage(&'static str),
}

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
