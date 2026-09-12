use tonic::{Request, Status};

use crate::grpc::auth;

pub fn auth_interceptor(mut req: Request<()>) -> Result<Request<()>, Status> {
    let token = req
        .metadata()
        .get("x-worker-token")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| Status::unauthenticated("missing worker token"))?;

    let claims = auth::validate_jwt(token)?;
    let worker = auth::AuthenticatedWorker::new(claims)?;

    req.extensions_mut().insert(worker);

    Ok(req)
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use serde::Serialize;
    use tonic::{Code, Request};

    use super::*;
    use crate::{grpc::auth::AuthenticatedWorker, proto::qer};

    #[derive(Serialize)]
    struct TestClaims {
        sub: String,
        scopes: Vec<i32>,
        iss: String,
        aud: Vec<String>,
        exp: usize,
        iat: usize,
        nbf: usize,
        jti: String,
    }

    fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    fn token(sub: &str, scopes: Vec<qer::v1::Scope>, iss: &str, aud: &str, exp_offset: i64) -> String {
        let claims = TestClaims {
            sub: sub.to_owned(),
            scopes: scopes.into_iter().map(i32::from).collect(),
            iss: iss.to_owned(),
            aud: vec![aud.to_owned()],
            exp: (now() + exp_offset) as usize,
            iat: now() as usize,
            nbf: now() as usize,
            jti: "test-jti".to_owned(),
        };
        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(b"secret"),
        )
        .unwrap()
    }

    fn request_with_token(token: Option<&str>) -> Request<()> {
        let mut request = Request::new(());
        if let Some(token) = token {
            request
                .metadata_mut()
                .insert("x-worker-token", token.parse().unwrap());
        }
        request
    }

    #[test]
    fn missing_token_header_is_rejected() {
        let request = request_with_token(None);
        let err = auth_interceptor(request).unwrap_err();
        assert_eq!(err.code(), Code::Unauthenticated);
    }

    #[test]
    fn malformed_token_is_rejected() {
        let request = request_with_token(Some("not-a-jwt"));
        let err = auth_interceptor(request).unwrap_err();
        assert_eq!(err.code(), Code::Unauthenticated);
    }

    #[test]
    fn expired_token_is_rejected() {
        let t = token(
            "worker-1",
            vec![qer::v1::Scope::QueueConsume],
            "qer-api",
            "qer-engine",
            -3600,
        );
        let request = request_with_token(Some(&t));

        let err = auth_interceptor(request).unwrap_err();
        assert_eq!(err.code(), Code::Unauthenticated);
    }

    #[test]
    fn wrong_issuer_is_rejected() {
        let t = token(
            "worker-1",
            vec![qer::v1::Scope::QueueConsume],
            "someone-else",
            "qer-engine",
            3600,
        );
        let request = request_with_token(Some(&t));

        let err = auth_interceptor(request).unwrap_err();
        assert_eq!(err.code(), Code::Unauthenticated);
    }

    #[test]
    fn valid_token_attaches_the_authenticated_worker() {
        let t = token(
            "worker-1",
            vec![qer::v1::Scope::QueueConsume, qer::v1::Scope::QueueProduce],
            "qer-api",
            "qer-engine",
            3600,
        );
        let request = request_with_token(Some(&t));

        let request = auth_interceptor(request).unwrap();
        let worker = request.extensions().get::<AuthenticatedWorker>().unwrap();

        assert_eq!(worker.worker_id.as_str(), "worker-1");
        assert_eq!(
            worker.scopes,
            vec![qer::v1::Scope::QueueConsume, qer::v1::Scope::QueueProduce]
        );
    }
}
