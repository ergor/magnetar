
#[derive(Debug)]
pub enum Error {
    Database(rusqlite::Error),
    Filesystem,
    IO,
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
