pub mod db;
pub mod model;

pub use db::AuthDb;
pub use model::UserInfo;

#[derive(Debug, thiserror::Error)]
pub enum Error<E: std::error::Error> {
    #[error("AuthDb error: {0}")]
    AuthDb(E),
    #[error("Custom OAuth 2.0 error: {0}")]
    Custom(String),
}
