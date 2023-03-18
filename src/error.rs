use rocket::{
    http::Status,
    request::Request,
    response::{Responder, Result},
    serde::json::Json,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("OAuth 2.0 error")]
    Oauth2(#[from] rocket_oauth2::Error),
    #[error("Authorization error")]
    Authorization(#[from] webb_auth::Error<webb_auth_sled::Error>),
    #[error("Twitter error")]
    TwitterError(#[from] twitter_v2::error::Error),
    #[error("Custom error")]
    Custom(String),
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ErrorResponse {
    pub status: &'static str,
    pub message: String,
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, req: &'r Request<'_>) -> Result<'o> {
        match self {
            Error::Oauth2(e) => Json(ErrorResponse {
                status: "oauth2 error",
                message: e.to_string(),
            })
            .respond_to(req),
            Error::Authorization(_) => Status::InternalServerError.respond_to(req),
            Error::TwitterError(e) => match e {
                twitter_v2::Error::Url(e) => Json(ErrorResponse {
                    status: "url error",
                    message: e.to_string(),
                })
                .respond_to(req),
                twitter_v2::Error::InvalidAuthorizationHeader(_) => {
                    Status::Unauthorized.respond_to(req)
                }
                twitter_v2::Error::NoRefreshToken => Status::NotFound.respond_to(req),
                twitter_v2::Error::Custom(e) => Json(ErrorResponse {
                    status: "custom error",
                    message: e.to_string(),
                })
                .respond_to(req),
                _ => Status::InternalServerError.respond_to(req),
            },
            Error::Custom(_) => Json(ErrorResponse {
                status: "custom error",
                message: self.to_string(),
            })
            .respond_to(req),
        }
    }
}
