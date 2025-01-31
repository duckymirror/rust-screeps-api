//! Interpreting login responses.
use std::borrow::Cow;

use crate::data;
use crate::error::{ApiError, Result};

use crate::{EndpointResult, Token, TokenStorage};

/// Login details
#[derive(Serialize, Clone, Hash, Debug)]
pub struct LoginArgs<'a> {
    /// The email or username to log in with (either works)
    email: Cow<'a, str>,
    /// The password to log in with (steam auth is not supported)
    password: Cow<'a, str>,
}

impl<'a> LoginArgs<'a> {
    /// Create a new login details with the given username and password
    pub fn new<T, U>(email: T, password: U) -> Self
    where
        T: Into<Cow<'a, str>>,
        U: Into<Cow<'a, str>>,
    {
        LoginArgs {
            email: email.into(),
            password: password.into(),
        }
    }
}

/// Login raw result.
#[derive(serde_derive::Deserialize, Clone, Hash, Debug)]
pub(crate) struct Response {
    ok: i32,
    token: Option<String>,
}

/// The result of a call to log in.
#[must_use = "LoggedIn does not do anything unless registered in a token store"]
#[derive(Clone, Hash, Debug)]
pub struct LoggedIn {
    /// The token which can be used to make future authenticated API calls.
    pub token: Token,
    /// Phantom data in order to allow adding any additional fields in the future.
    _non_exhaustive: (),
}

impl LoggedIn {
    /// Stores the token into the given token storage.
    pub fn return_to(self, storage: &TokenStorage) {
        storage.set(self.token);
    }
}

impl EndpointResult for LoggedIn {
    type RequestResult = Response;
    type ErrorResult = data::ApiError;

    fn from_raw(raw: Response) -> Result<LoggedIn> {
        let Response { ok, token } = raw;

        if ok != 1 {
            return Err(ApiError::NotOk(ok).into());
        }
        match token {
            Some(token) => Ok(LoggedIn {
                token: token.into(),
                _non_exhaustive: (),
            }),
            None => Err(ApiError::MissingField("token").into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LoggedIn;
    use crate::EndpointResult;
    use serde_json;

    fn test_parse(json: serde_json::Value) {
        let response = serde_json::from_value(json).unwrap();

        let _ = LoggedIn::from_raw(response).unwrap();
    }

    #[test]
    fn parse_sample_login_success() {
        test_parse(json! ({
            "ok": 1,
            "token": "c07924d3f556a355eba7cd59f4c21f670fda76c2",
        }));
    }
}
