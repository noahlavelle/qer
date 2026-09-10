use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, errors::ErrorKind};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tonic::Request;

use crate::engine::WorkerID;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("token is invalid")]
    InvalidToken,
    #[error("token has expired")]
    TokenExpired,
    #[error("token is not yet valid")]
    TokenNotYetValid,
    #[error("token signature is invalid")]
    InvalidSignature,
    #[error("token issuer is invalid")]
    InvalidIssuer,
    #[error("token audience is invalid")]
    InvalidAudience,
    #[error("token is missing required claim: {0}")]
    MissingClaim(String),
    #[error("missing authenticated worker")]
    NotAuthed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkerClaims {
    sub: String,
    scopes: Vec<String>,
    iss: String,
    aud: String,
    exp: usize,
    iat: usize,
    nbf: usize,
    jti: String,
}

#[derive(Clone)]
pub struct AuthenticatedWorker {
    pub worker_id: WorkerID,
    pub scopes: Vec<String>,
}

impl AuthenticatedWorker {
    pub fn new(claims: WorkerClaims) -> Result<Self, AuthError> {
        let worker_id =
            WorkerID::new(claims.sub).map_err(|_| AuthError::MissingClaim("sub".to_owned()))?;

        if claims.scopes.is_empty() {
            return Err(AuthError::MissingClaim("scopes".to_owned()));
        }

        Ok(Self {
            worker_id,
            scopes: claims.scopes,
        })
    }
}

pub fn validate_jwt(token: &str) -> Result<WorkerClaims, AuthError> {
    let key = DecodingKey::from_secret(b"secret");

    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&["qer-api"]);
    validation.set_audience(&["qer-engine"]);

    let token_data =
        decode::<WorkerClaims>(token, &key, &validation).map_err(|err| match err.kind() {
            ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            ErrorKind::ImmatureSignature => AuthError::TokenNotYetValid,
            ErrorKind::InvalidSignature => AuthError::InvalidSignature,
            ErrorKind::InvalidIssuer => AuthError::InvalidIssuer,
            ErrorKind::InvalidAudience => AuthError::InvalidAudience,
            ErrorKind::MissingRequiredClaim(claim) => AuthError::MissingClaim(claim.clone()),
            _ => AuthError::InvalidToken,
        })?;

    Ok(token_data.claims)
}

pub fn auth_from_request<R>(req: &Request<R>) -> Result<AuthenticatedWorker, AuthError> {
    req.extensions()
        .get::<AuthenticatedWorker>()
        .cloned()
        .ok_or(AuthError::NotAuthed)
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use jsonwebtoken::{EncodingKey, Header, encode};

    use super::*;

    fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    fn make_claims(sub: &str, scopes: Vec<&str>, iss: &str, aud: &str, exp_offset: i64) -> WorkerClaims {
        WorkerClaims {
            sub: sub.to_owned(),
            scopes: scopes.into_iter().map(String::from).collect(),
            iss: iss.to_owned(),
            aud: aud.to_owned(),
            exp: (now() + exp_offset) as usize,
            iat: now() as usize,
            nbf: now() as usize,
            jti: "test-jti".to_owned(),
        }
    }

    fn token_signed_with(claims: &WorkerClaims, secret: &[u8]) -> String {
        encode(&Header::new(Algorithm::HS256), claims, &EncodingKey::from_secret(secret)).unwrap()
    }

    fn valid_token() -> String {
        let claims = make_claims("worker-1", vec!["reserve", "ack"], "qer-api", "qer-engine", 3600);
        token_signed_with(&claims, b"secret")
    }

    /// Like `Result::unwrap_err`, but doesn't require the `Ok` variant to be `Debug`.
    fn expect_err<T, E>(result: Result<T, E>) -> E {
        match result {
            Err(e) => e,
            Ok(_) => panic!("expected an error"),
        }
    }

    #[test]
    fn validate_jwt_accepts_a_valid_token() {
        let claims = validate_jwt(&valid_token()).unwrap();
        assert_eq!(claims.sub, "worker-1");
        assert_eq!(claims.scopes, vec!["reserve", "ack"]);
    }

    #[test]
    fn validate_jwt_rejects_wrong_issuer() {
        let claims = make_claims("worker-1", vec!["reserve"], "someone-else", "qer-engine", 3600);
        let token = token_signed_with(&claims, b"secret");

        let err = expect_err(validate_jwt(&token));
        assert!(matches!(err, AuthError::InvalidIssuer));
    }

    #[test]
    fn validate_jwt_rejects_wrong_audience() {
        let claims = make_claims("worker-1", vec!["reserve"], "qer-api", "someone-else", 3600);
        let token = token_signed_with(&claims, b"secret");

        let err = expect_err(validate_jwt(&token));
        assert!(matches!(err, AuthError::InvalidAudience));
    }

    #[test]
    fn validate_jwt_rejects_expired_token() {
        let claims = make_claims("worker-1", vec!["reserve"], "qer-api", "qer-engine", -3600);
        let token = token_signed_with(&claims, b"secret");

        let err = expect_err(validate_jwt(&token));
        assert!(matches!(err, AuthError::TokenExpired));
    }

    #[test]
    fn validate_jwt_rejects_a_bad_signature() {
        let claims = make_claims("worker-1", vec!["reserve"], "qer-api", "qer-engine", 3600);
        let token = token_signed_with(&claims, b"not-the-real-secret");

        let err = expect_err(validate_jwt(&token));
        assert!(matches!(err, AuthError::InvalidSignature));
    }

    #[test]
    fn validate_jwt_rejects_garbage_input() {
        assert!(validate_jwt("not-a-jwt").is_err());
    }

    #[test]
    fn authenticated_worker_new_succeeds_with_valid_claims() {
        let claims = make_claims("worker-1", vec!["reserve"], "qer-api", "qer-engine", 3600);
        let worker = AuthenticatedWorker::new(claims).unwrap();

        assert_eq!(worker.worker_id.as_str(), "worker-1");
        assert_eq!(worker.scopes, vec!["reserve"]);
    }

    #[test]
    fn authenticated_worker_new_rejects_empty_scopes() {
        let claims = make_claims("worker-1", vec![], "qer-api", "qer-engine", 3600);
        let err = expect_err(AuthenticatedWorker::new(claims));
        assert!(matches!(err, AuthError::MissingClaim(claim) if claim == "scopes"));
    }

    #[test]
    fn authenticated_worker_new_rejects_empty_sub() {
        let claims = make_claims("", vec!["reserve"], "qer-api", "qer-engine", 3600);
        let err = expect_err(AuthenticatedWorker::new(claims));
        assert!(matches!(err, AuthError::MissingClaim(claim) if claim == "sub"));
    }

    #[test]
    fn auth_from_request_errors_without_an_authenticated_worker() {
        let request = Request::new(());
        let err = expect_err(auth_from_request(&request));
        assert!(matches!(err, AuthError::NotAuthed));
    }

    #[test]
    fn auth_from_request_returns_the_inserted_worker() {
        let mut request = Request::new(());
        request.extensions_mut().insert(AuthenticatedWorker {
            worker_id: WorkerID::new("worker-1").unwrap(),
            scopes: vec!["reserve".to_owned()],
        });

        let worker = auth_from_request(&request).unwrap();
        assert_eq!(worker.worker_id.as_str(), "worker-1");
        assert_eq!(worker.scopes, vec!["reserve"]);
    }
}
