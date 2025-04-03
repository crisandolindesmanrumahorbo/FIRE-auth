use std::{error::Error, fmt::Debug};

#[derive(thiserror::Error)]
pub enum CustomError {
    #[error("ENV '{0}' Not Found")]
    EnvError(String, #[source] std::env::VarError),

    #[error("Error encode private key")]
    EncodeError(#[source] jsonwebtoken::errors::Error),

    #[error("Database error")]
    DBError(#[from] sqlx::Error),
    
    #[error("User not found")]
    UserNotFound,

    #[error("Username already exists")]
    UsernameExists,
}

impl Debug for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)?;
        if let Some(source) = self.source() {
            write!(f, " (Caused by: {})", source)?;
        }
        Ok(())
    }
}

// code above equivalent with this
// pub enum CustomError {
//     EnvError(String, std::env::VarError),
//     EncodeError(jsonwebtoken::errors::Error),
//     DBConnectionError(postgres::Error),
//     DBQueryError(postgres::Error),
// }

// impl std::error::Error for CustomError {
//     fn source(&self) -> Option<&(dyn Error + 'static)> {
//         match self {
//             CustomError::DBConnectionError(s) => Some(s),
//             CustomError::DBQueryError(s) => Some(s),
//             CustomError::EncodeError(s) => Some(s),
//             CustomError::EnvError(_,e) => Some(e),
//         }
//     }
// }

// impl Debug for CustomError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         writeln!(f, "{}", self)?;
//         if let Some(source) = self.source() {
//             writeln!(f, "Caused by:\n\t{}", source)?;
//         }
//         Ok(())
//     }
// }

// impl std::fmt::Display for CustomError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             CustomError::DBConnectionError(_) => write!(f, "failed to read the key file"),
//             CustomError::DBQueryError(_) => write!(f, "failed to send the api request"),
//             CustomError::EncodeError(_) => write!(f, "failed to delete the key file"),
//             CustomError::EnvError(_, _) => write!(f, "failed to delete the key file"),
//         }
//     }
// }
