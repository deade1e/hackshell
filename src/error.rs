use std::fmt::Display;

use rustyline::error::ReadlineError;

#[derive(Debug)]
pub enum HackshellError {
    String(String),
    Generic(Box<dyn std::error::Error + Send + Sync + 'static>),
    ShellExit,
    Interrupted,
    Eof,
    OtherReadline(ReadlineError),
}

impl From<Box<dyn std::error::Error + Send + Sync + 'static>> for HackshellError {
    fn from(value: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self {
        // Like when Yagami Raito forgot being Kira only to remember it afterwards
        match value.downcast::<HackshellError>() {
            Ok(e) => *e,
            Err(e) => Self::Generic(e),
        }
    }
}

impl From<String> for HackshellError {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for HackshellError {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<ReadlineError> for HackshellError {
    fn from(value: ReadlineError) -> Self {
        match value {
            ReadlineError::Interrupted => Self::Interrupted,
            ReadlineError::Eof => Self::Eof,
            _ => Self::OtherReadline(value),
        }
    }
}

impl Display for HackshellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OtherReadline(string) => write!(f, "Readline error: {}", string),
            Self::String(string) => write!(f, "{}", string),
            Self::Generic(e) => write!(f, "{}", e),
            Self::ShellExit => write!(f, "Shell exit"),
            Self::Interrupted => write!(f, "Interrupted"),
            Self::Eof => write!(f, "EOF"),
        }
    }
}

impl std::error::Error for HackshellError {}

pub type Result<T> = std::result::Result<T, HackshellError>;
