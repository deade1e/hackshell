use std::{any::Any, fmt::Display};

use rustyline::error::ReadlineError;

#[derive(Debug)]
pub enum JoinError {
    Sync(Box<dyn Any + Sync + Send + 'static>),

    AlreadyJoining,
    #[cfg(feature = "async")]
    CannotJoinAsync,
    #[cfg(feature = "async")]
    Async(tokio::task::JoinError),
}

#[derive(Debug)]
pub enum HackshellError {
    String(String),
    Generic(Box<dyn std::error::Error + Send + Sync + 'static>),
    CommandNotFound,
    TaskNotFound,
    Exit,
    Interrupted,
    Eof,
    OtherReadline(ReadlineError),
    JoinError(JoinError),
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

#[cfg(feature = "async")]
impl From<tokio::task::JoinError> for HackshellError {
    fn from(value: tokio::task::JoinError) -> Self {
        // Conversion loses context
        Self::JoinError(JoinError::Async(value))
    }
}

impl Display for HackshellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OtherReadline(message) => write!(f, "Readline error: {}", message),
            Self::String(message) => write!(f, "{}", message),
            Self::Generic(e) => write!(f, "{}", e),
            Self::CommandNotFound => write!(f, "Command not found"),
            Self::TaskNotFound => write!(f, "Task not found"),
            Self::Exit => write!(f, "Shell exit"),
            Self::Interrupted => write!(f, "Interrupted"),
            Self::Eof => write!(f, "EOF"),
            Self::JoinError(e) => match e {
                JoinError::Sync(e) => {
                    if let Some(message) = e.downcast_ref::<&str>() {
                        write!(f, "Thread panicked: {}", message)
                    } else if let Some(message) = e.downcast_ref::<String>() {
                        write!(f, "Thread panicked: {}", message)
                    } else {
                        write!(f, "Thread panicked with non-string payload")
                    }
                }
                JoinError::AlreadyJoining => write!(f, "Task has been already joined"),
                #[cfg(feature = "async")]
                JoinError::Async(e) => write!(f, "{}", e),
                #[cfg(feature = "async")]
                JoinError::CannotJoinAsync => {
                    write!(f, "Sync task cannot be joined asynchronously")
                }
            },
        }
    }
}

impl std::error::Error for HackshellError {}

pub type HackshellResult<T> = std::result::Result<T, HackshellError>;
