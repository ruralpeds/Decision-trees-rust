use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use crate::config::JwtClaims;
use crate::error::AppError;

/// Extractor for JWT authentication
/// Usage in handlers:
/// ```ignore
/// async fn my_handler(claims: JwtClaims, ...) -> ApiResult<Json<T>> {
///     // JWT is already validated
///     println!("User: {}", claims.sub);
/// }
/// ```
#[async_trait]
impl<S> FromRequestParts<S> for JwtClaims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or(AppError::AuthenticationFailed(
                "Missing Authorization header".to_string(),
            ))?;

        // Extract bearer token
        let token = if auth_header.starts_with("Bearer ") {
            &auth_header[7..]
        } else {
            return Err(AppError::AuthenticationFailed(
                "Invalid Authorization header format".to_string(),
            ));
        };

        // Decode JWT (without verification for now; in production, verify signature)
        decode_token(token)
    }
}

/// Decode and validate JWT token
fn decode_token(token: &str) -> Result<JwtClaims, AppError> {
    // In production, load the public key from config
    // For now, use a simple algorithm that doesn't verify signature
    // This is for development only

    // Attempt to decode without verification
    let token_data = jsonwebtoken::decode::<JwtClaims>(
        token,
        // In production, use actual public key:
        // let key = DecodingKey::from_rsa_pem(config.jwt_public_key.as_bytes())?;
        &DecodingKey::from_secret(b"secret"),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|e| AppError::AuthenticationFailed(format!("Invalid token: {}", e)))?;

    Ok(token_data.claims)
}

/// Extract roles from JWT claims
pub fn extract_roles(claims: &JwtClaims) -> Vec<String> {
    claims.roles.clone()
}

/// Check if user has a required role
pub fn has_role(claims: &JwtClaims, required_role: &str) -> bool {
    claims.roles.iter().any(|r| r == required_role)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_roles() {
        let claims = JwtClaims {
            sub: "user-123".to_string(),
            iss: "auth.example.com".to_string(),
            exp: 9999999999,
            iat: 0,
            roles: vec!["clinician".to_string(), "author".to_string()],
            org_id: None,
        };

        let roles = extract_roles(&claims);
        assert_eq!(roles.len(), 2);
        assert!(roles.contains(&"clinician".to_string()));
    }

    #[test]
    fn test_has_role() {
        let claims = JwtClaims {
            sub: "user-123".to_string(),
            iss: "auth.example.com".to_string(),
            exp: 9999999999,
            iat: 0,
            roles: vec!["clinician".to_string()],
            org_id: None,
        };

        assert!(has_role(&claims, "clinician"));
        assert!(!has_role(&claims, "admin"));
    }
}
