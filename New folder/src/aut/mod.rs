use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use axum::http::{Request, header};

pub mod jwt;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub roles: Vec<String>,
}

pub fn generate_token(user_id: &str, roles: Vec<String>, secret: &str) -> anyhow::Result<String> {
    let now = Utc::now();
    let expire = now + Duration::hours(24);
    
    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expire.timestamp() as usize,
        iat: now.timestamp() as usize,
        roles,
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| anyhow::anyhow!("Token generation failed: {}", e))
}