use std::fmt;

/// An error type for all in-house funcs & methods.
#[derive(Debug)]
pub enum AppError {
    /// For use when an `Option` was `None`, but `Some(a)` was expected.
    /// Workaround until the `Try` trait becomes stable.
    NoneError,

    WithMessage(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let buf: String = match self {
            AppError::NoneError => {
                format!("{}", "AppError::NoneError: An 'Option::None' was unwrapped.")
            },
            AppError::WithMessage(msg) => {
                format!("AppError::WithMessage: '{}'", msg)
            }
        };
        write!(f, "{}", buf)
    }
}