use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use axum::http::{Request, header};
use axum::response::{Response, IntoResponse};
use axum::http::StatusCode;

pub fn validate_token(req: &Request<axum::body::Body>, secret: &str) -> anyhow::Result<()> {
    let auth_header = req.headers()
        .get(header::AUTHORIZATION)
        .ok_or_else(|| anyhow::anyhow!("Missing authorization header"))?;
    
    let auth_str = auth_header.to_str()
        .map_err(|_| anyhow::anyhow!("Invalid authorization header"))?;
    
    if !auth_str.starts_with("Bearer ") {
        return Err(anyhow::anyhow!("Invalid authorization scheme"));
    }
    
    let token = &auth_str[7..];
    
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);
    
    decode::<crate::auth::Claims>(token, &decoding_key, &validation)
        .map_err(|e| anyhow::anyhow!("Invalid token: {}", e))?;
    
    Ok(())
}

pub async fn auth_middleware<B>(req: Request<B>, next: impl FnOnce(Request<B>) -> Response) -> Response {
    // Simplified auth middleware
    // Full implementation would check config and validate tokens
    next(req)
}