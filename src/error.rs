//! Error types and conversion functions.

use std::error::Error;
use std::fmt;

/// An enum containing all kinds of game framework errors.
#[derive(Debug)]
pub enum GameError {
    /// Something went wrong trying to read from a file
    IOError(std::io::Error),
    UnknownError(String),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            _ => write!(f, "GameError {:?}", self),
        }
    }
}

impl Error for GameError {
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            GameError::IOError(ref e) => Some(e),
            _ => None,
        }
    }
}

/// A convenient result type consisting of a return type and a `GameError`
pub type GameResult<T = ()> = Result<T, GameError>;

impl From<std::io::Error> for GameError {
    fn from(e: std::io::Error) -> GameError {
        GameError::IOError(e)
    }
}
