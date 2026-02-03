use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claim {
    pub sub: String,
    pub exp: usize,
}
const jwt_secret: &[u8] = b"yoyoyo";
pub fn create_jwt(userid: &str) -> String {
    let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;
    let claim = Claim {
        sub: userid.to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(jwt_secret),
    )
    .expect("jwt creation failed")
}
pub fn verify(token: &str) -> Result<Claim, jsonwebtoken::errors::Error> {
    let data = decode(
        token,
        &DecodingKey::from_secret(jwt_secret),
        &Validation::default(),
    )?;
    Ok(data.claims)
}
