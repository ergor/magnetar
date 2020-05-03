
use std::{io, time};

#[derive(Debug)]
pub enum Error {
    Database(rusqlite::Error),
    Filesystem,
    IO(io::Error),
    NoneError,
    SystemTimeError(time::SystemTimeError),
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Error {
        Error::Database(e)
    }
}

impl From<(rusqlite::Connection, rusqlite::Error)> for Error {
    fn from(e: (rusqlite::Connection, rusqlite::Error)) -> Error {
        let (conn, error) = e;
        Error::Database(error)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}

impl From<time::SystemTimeError> for Error {
    fn from(e: time::SystemTimeError) -> Error {
        Error::SystemTimeError(e)
    }
}
