//! Error types and conversion functions.

use std::error::Error;
use std::fmt;

/// An enum containing all kinds of game framework errors.
#[derive(Debug)]
pub enum GameError {
    /// Something went wrong trying to read from a file
    IOError(std::io::Error),
    /// Something went wrong compiling shaders
    ShaderProgramError(String),
    /// Something went wrong with the `gilrs` gamepad-input library.
    GamepadError(String),
    /// Something went wrong with the `lyon` shape-tesselation library
    LyonError(String),
    /// SoundMixer in the context should be created explicitly from some of the interaction callbacks
    /// Thats the only way to get audio to works on web :(
    MixerNotCreated,
    SoundError,
    UnknownError(String),
    /// Unable to find a resource; the `Vec` is the paths it searched for and associated errors
    ResourceNotFound(String, Vec<(std::path::PathBuf, GameError)>),
    /// An error in the filesystem layout
    FilesystemError(String),
    /// An error trying to load a resource, such as getting an invalid image file.
    ResourceLoadError(String),
    /// Something went wrong in the renderer
    RenderError(String),
    /// A custom error type for use by users of ggez.
    /// This lets you handle custom errors that may happen during your game (such as, trying to load a malformed file for a level)
    /// using the same mechanism you handle ggez's other errors.
    ///
    /// Please include an informative message with the error.
    CustomError(String),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GameError {:?}", self)
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
/*
impl From<miniquad_text_rusttype::Error> for GameError {
    fn from(e: miniquad_text_rusttype::Error) -> GameError {
        GameError::TTFError(e)
    }
}
*/
#[cfg(feature = "mesh")]
impl From<lyon::lyon_tessellation::TessellationError> for GameError {
    fn from(s: lyon::lyon_tessellation::TessellationError) -> GameError {
        let errstr = format!(
            "Error while tesselating shape (did you give it an infinity or NaN?): {:?}",
            s
        );
        GameError::LyonError(errstr)
    }
}

#[cfg(feature = "mesh")]
impl From<lyon::lyon_tessellation::geometry_builder::GeometryBuilderError> for GameError {
    fn from(s: lyon::lyon_tessellation::geometry_builder::GeometryBuilderError) -> GameError {
        let errstr = format!(
            "Error while building geometry (did you give it too many vertices?): {:?}",
            s
        );
        GameError::LyonError(errstr)
    }
}

impl From<zip::result::ZipError> for GameError {
    fn from(e: zip::result::ZipError) -> GameError {
        let errstr = format!("Zip error: {}", e.to_string());
        GameError::ResourceLoadError(errstr)
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "ios", target_os = "android",)))]
impl From<gilrs::Error> for GameError {
    fn from(s: gilrs::Error) -> GameError {
        let errstr = format!("Gamepad error: {}", s);
        GameError::GamepadError(errstr)
    }
}

impl From<miniquad::ShaderError> for GameError {
    fn from(e: miniquad::ShaderError) -> GameError {
        let errstr = format!("Shader creation error: {}", e);
        GameError::ShaderProgramError(errstr)
    }
}
