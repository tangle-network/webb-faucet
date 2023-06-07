use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};

pub mod login;

/// Contains the OAuth2 provider implementations.
pub mod providers {

    pub trait Provider {
        const NAME: &'static str;
        fn name() -> &'static str {
            Self::NAME
        }
    }
    /// Twitter OAuth2 provider.
    #[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Twitter;

    impl Provider for Twitter {
        const NAME: &'static str = "twitter";
    }
}

/// Twitter-specific OAuth2 Authurization token.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TwitterBearerToken<'r>(&'r str);

impl<'r> TwitterBearerToken<'r> {
    pub fn token(&self) -> &'r str {
        self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TwitterBearerTokenError {
    Missing,
    Malformed,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for TwitterBearerToken<'r> {
    type Error = TwitterBearerTokenError;

    async fn from_request(
        request: &'r rocket::Request<'_>,
    ) -> Outcome<Self, Self::Error> {
        let maybe_token = match request.headers().get_one("Authorization") {
            Some(token) => token.trim(),
            None => {
                return Outcome::Failure((
                    Status::Unauthorized,
                    TwitterBearerTokenError::Missing,
                ))
            }
        };
        let token = match maybe_token.strip_prefix("Bearer ") {
            Some(token) => token,
            None => {
                return Outcome::Failure((
                    Status::Unauthorized,
                    TwitterBearerTokenError::Malformed,
                ))
            }
        };
        Outcome::Success(TwitterBearerToken(token))
    }
}
