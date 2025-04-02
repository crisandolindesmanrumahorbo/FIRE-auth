
use std::{error::Error, fmt::Debug};

#[derive(thiserror::Error)]
pub enum CustomError {
    #[error("ENV '{0}' Not Found")]
    EnvError(String, #[source] std::env::VarError),

    #[error("Error encode private key")]
    EncodeError(#[source] jsonwebtoken::errors::Error),

    #[error("Database connection")]
    DBConnectionError(#[source] postgres::Error),

    #[error("Database query")]
    DBQueryError(#[source] postgres::Error),
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