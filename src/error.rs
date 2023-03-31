use rocket::{
    request::Request,
    response::{Responder, Result},
    serde::json::Json,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("OAuth 2.0 error: {0}")]
    Oauth2(#[from] rocket_oauth2::Error),
    #[error("Database error: {0}")]
    AuthDatabase(#[from] webb_auth_sled::Error),
    #[error("Twitter error: {0}")]
    TwitterError(#[from] twitter_v2::error::Error),
    #[error("Custom error: {0}")]
    Custom(String),
}

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum FaucetErrorCode {
    /// An error occurred during URI parsing or construction. This usually means
    /// the token exchange endpoint is incorrect. The attempted URI is included.
    Oauth2InvalidUri = 100000,
    /// A token exchange request failed, for example because the server could
    /// not be reached, or the response body could not be parsed.
    Oauth2ExchangeFailure = 100001,
    /// An unknown error occurred during token exchange.
    Oauth2Unknown = 100002,
    /// A Database error occurred.
    DatabaseError = 200000,
    /// A Data model serialization error occurred.
    DataSerializationError = 200001,
    /// An error occurred while communicating with Twitter API.
    TwitterApiError = 300000,
    /// An Unknown error occurred.
    CustomError = 400000,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ErrorResponse {
    pub code: FaucetErrorCode,
    pub message: String,
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, req: &'r Request<'_>) -> Result<'o> {
        match self {
            Error::Oauth2(ref err) => match err.kind() {
                rocket_oauth2::ErrorKind::InvalidUri(_) => {
                    let response = ErrorResponse {
                        code: FaucetErrorCode::Oauth2InvalidUri,
                        message: self.to_string(),
                    };
                    Json(response).respond_to(req)
                }
                rocket_oauth2::ErrorKind::ExchangeFailure
                | rocket_oauth2::ErrorKind::ExchangeError(_) => {
                    let response = ErrorResponse {
                        code: FaucetErrorCode::Oauth2ExchangeFailure,
                        message: self.to_string(),
                    };
                    Json(response).respond_to(req)
                }
                rocket_oauth2::ErrorKind::Other => {
                    let response = ErrorResponse {
                        code: FaucetErrorCode::Oauth2Unknown,
                        message: self.to_string(),
                    };
                    Json(response).respond_to(req)
                }
            },
            Error::AuthDatabase(ref err) => match err {
                webb_auth_sled::Error::Sled(_) => {
                    let response = ErrorResponse {
                        code: FaucetErrorCode::DatabaseError,
                        message: self.to_string(),
                    };
                    Json(response).respond_to(req)
                }
                webb_auth_sled::Error::InvalidId(_) => {
                    let response = ErrorResponse {
                        code: FaucetErrorCode::DataSerializationError,
                        message: self.to_string(),
                    };
                    Json(response).respond_to(req)
                }
                webb_auth_sled::Error::Serde(_) => {
                    let response = ErrorResponse {
                        code: FaucetErrorCode::DataSerializationError,
                        message: self.to_string(),
                    };
                    Json(response).respond_to(req)
                }
            },
            Error::TwitterError(_) => {
                let response = ErrorResponse {
                    code: FaucetErrorCode::TwitterApiError,
                    message: self.to_string(),
                };
                Json(response).respond_to(req)
            }
            Error::Custom(_) => {
                let response = ErrorResponse {
                    code: FaucetErrorCode::CustomError,
                    message: self.to_string(),
                };
                Json(response).respond_to(req)
            }
        }
    }
}
