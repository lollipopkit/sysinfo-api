use axum::{
    extract::{Request, State},
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use base64::{Engine as _, engine::general_purpose};
use hyper::{StatusCode, header};
use subtle::ConstantTimeEq;

use crate::AuthState;

// Basic auth middleware
pub async fn basic_auth(
    State(auth_state): State<AuthState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = match auth_header {
        Some(header) => header,
        None => return create_auth_error_response(),
    };

    let credentials = match auth_header.strip_prefix("Basic ") {
        Some(creds) => creds,
        None => return create_auth_error_response(),
    };

    let decoded = match general_purpose::STANDARD.decode(credentials) {
        Ok(decoded) => decoded,
        Err(_) => return create_auth_error_response(),
    };

    let credentials_str = match String::from_utf8(decoded) {
        Ok(s) => s,
        Err(_) => return create_auth_error_response(),
    };

    // Use constant-time comparison to prevent timing attacks
    if credentials_str
        .as_bytes()
        .ct_eq(auth_state.expected_credentials.as_bytes())
        .into()
    {
        Ok(next.run(req).await)
    } else {
        create_auth_error_response()
    }
}

fn create_auth_error_response() -> Result<Response, StatusCode> {
    let mut response = Response::new(axum::body::Body::empty());
    *response.status_mut() = StatusCode::UNAUTHORIZED;
    response.headers_mut().insert(
        header::WWW_AUTHENTICATE,
        HeaderValue::from_static("Basic realm=\"API Access\""),
    );
    Ok(response)
}
