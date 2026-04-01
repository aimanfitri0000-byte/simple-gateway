// src/auth/mod.rs
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

const JWT_SECRET: &[u8] = b"your-secret-key-change-in-production";

#[derive(Debug, Serialize, Deserialize, Clone)] // ← Clone diperlukan
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn generate_token(user_id: &str) -> String {
    let now = Utc::now();
    let expire = now + Duration::hours(24);
    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expire.timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .unwrap()
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

lazy_static! {
    static ref USERS: Vec<(String, String)> = {
        let hash = hash("password123", DEFAULT_COST).unwrap();
        vec![("alice".to_string(), hash)]
    };
}

pub fn authenticate(username: &str, password: &str) -> Option<String> {
    for (user, hash) in USERS.iter() {
        if user == username && verify(password, hash).unwrap_or(false) {
            return Some(generate_token(username));
        }
    }
    None
}
