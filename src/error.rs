//! Error types and conversion functions.

use std::error::Error;
use std::fmt;

/// An enum containing all kinds of game framework errors.
#[derive(Debug)]
pub enum GameError {
    /// Something went wrong trying to read from a file
    IOError(std::io::Error),
    /// Something went wrong with the `lyon` shape-tesselation library
    LyonError(String),
    TTFError(miniquad_text_rusttype::Error),
    UnknownError(&'static str),
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

impl From<miniquad_text_rusttype::Error> for GameError {
    fn from(e: miniquad_text_rusttype::Error) -> GameError {
        GameError::TTFError(e)
    }
}

#[cfg(feature = "mesh")]
impl From<lyon::lyon_tessellation::FillError> for GameError {
    fn from(s: lyon::lyon_tessellation::FillError) -> GameError {
        let errstr = format!(
            "Error while tesselating shape (did you give it an infinity or NaN?): {:?}",
            s
        );
        GameError::LyonError(errstr)
    }
}
