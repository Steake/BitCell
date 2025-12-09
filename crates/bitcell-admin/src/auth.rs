//! Authentication and authorization for admin console
//!
//! Implements JWT-based authentication with role-based access control (RBAC).

use axum::{
    async_trait,
    extract::{FromRequestParts, Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
    Json,
};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;

/// User role for RBAC
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Full system access - can modify configuration, manage nodes, view all data
    Admin,
    /// Operational access - can start/stop nodes, view data, but cannot modify config
    Operator,
    /// Read-only access - can only view data and logs
    Viewer,
}

impl Role {
    /// Check if this role has permission for another role's actions
    pub fn can_perform(&self, required: Role) -> bool {
        match self {
            Role::Admin => true, // Admin can do everything
            Role::Operator => matches!(required, Role::Operator | Role::Viewer),
            Role::Viewer => matches!(required, Role::Viewer),
        }
    }
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
}

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // Subject (user id)
    pub username: String,
    pub role: Role,
    pub exp: i64,         // Expiry time
    pub iat: i64,         // Issued at
    pub jti: String,      // JWT ID (for token revocation)
}

/// Authentication request
#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Authentication response
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserInfo,
}

/// User info in response
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub role: Role,
}

/// Refresh token request
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Auth manager handles authentication and authorization
pub struct AuthManager {
    users: RwLock<Vec<User>>,
    revoked_tokens: RwLock<std::collections::HashSet<String>>,
    jwt_secret: EncodingKey,
    jwt_decoding: DecodingKey,
}

impl AuthManager {
    /// Create a new auth manager with a secret key
    pub fn new(secret: &str) -> Self {
        let jwt_secret = EncodingKey::from_secret(secret.as_bytes());
        let jwt_decoding = DecodingKey::from_secret(secret.as_bytes());
        
        // Create default admin user (password: "admin")
        // WARNING: In production, this should be changed immediately
        let default_admin = User {
            id: Uuid::new_v4().to_string(),
            username: "admin".to_string(),
            password_hash: bcrypt::hash("admin", bcrypt::DEFAULT_COST).unwrap(),
            role: Role::Admin,
            created_at: Utc::now(),
        };

        Self {
            users: RwLock::new(vec![default_admin]),
            revoked_tokens: RwLock::new(std::collections::HashSet::new()),
            jwt_secret,
            jwt_decoding,
        }
    }

    /// Authenticate user and generate tokens
    pub fn login(&self, req: LoginRequest) -> Result<AuthResponse, AuthError> {
        let users = self.users.read();
        let user = users
            .iter()
            .find(|u| u.username == req.username)
            .ok_or(AuthError::InvalidCredentials)?;

        // Verify password
        if !bcrypt::verify(&req.password, &user.password_hash)
            .map_err(|_| AuthError::InvalidCredentials)?
        {
            return Err(AuthError::InvalidCredentials);
        }

        // Generate access token (1 hour expiry)
        let access_token = self.generate_token(&user, 3600)?;
        
        // Generate refresh token (7 days expiry)
        let refresh_token = self.generate_token(&user, 604800)?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user: UserInfo {
                id: user.id.clone(),
                username: user.username.clone(),
                role: user.role,
            },
        })
    }

    /// Generate JWT token
    fn generate_token(&self, user: &User, expires_in: i64) -> Result<String, AuthError> {
        let now = Utc::now();
        let claims = Claims {
            sub: user.id.clone(),
            username: user.username.clone(),
            role: user.role,
            exp: (now + Duration::seconds(expires_in)).timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
        };

        encode(&Header::new(Algorithm::HS256), &claims, &self.jwt_secret)
            .map_err(|_| AuthError::TokenGenerationFailed)
    }

    /// Validate and decode JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        // Check if token is revoked
        if self.revoked_tokens.read().contains(token) {
            return Err(AuthError::TokenRevoked);
        }

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        decode::<Claims>(token, &self.jwt_decoding, &validation)
            .map(|data| data.claims)
            .map_err(|_| AuthError::InvalidToken)
    }

    /// Refresh access token using refresh token
    pub fn refresh(&self, req: RefreshRequest) -> Result<AuthResponse, AuthError> {
        let claims = self.validate_token(&req.refresh_token)?;
        
        let users = self.users.read();
        let user = users
            .iter()
            .find(|u| u.id == claims.sub)
            .ok_or(AuthError::UserNotFound)?;

        // Revoke old refresh token
        self.revoked_tokens.write().insert(req.refresh_token);

        // Generate new tokens
        let access_token = self.generate_token(user, 3600)?;
        let refresh_token = self.generate_token(user, 604800)?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user: UserInfo {
                id: user.id.clone(),
                username: user.username.clone(),
                role: user.role,
            },
        })
    }

    /// Revoke a token (for logout)
    pub fn revoke_token(&self, token: String) {
        self.revoked_tokens.write().insert(token);
    }

    /// Add a new user (admin only)
    pub fn add_user(&self, username: String, password: String, role: Role) -> Result<User, AuthError> {
        let mut users = self.users.write();
        
        // Check if user already exists
        if users.iter().any(|u| u.username == username) {
            return Err(AuthError::UserAlreadyExists);
        }

        let user = User {
            id: Uuid::new_v4().to_string(),
            username,
            password_hash: bcrypt::hash(password, bcrypt::DEFAULT_COST)
                .map_err(|_| AuthError::PasswordHashFailed)?,
            role,
            created_at: Utc::now(),
        };

        users.push(user.clone());
        Ok(user)
    }
}

/// Authentication errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token generation failed")]
    TokenGenerationFailed,
    #[error("Token has been revoked")]
    TokenRevoked,
    #[error("User not found")]
    UserNotFound,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Password hash failed")]
    PasswordHashFailed,
    #[error("Insufficient permissions")]
    InsufficientPermissions,
}

impl axum::response::IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthError::TokenRevoked => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthError::InsufficientPermissions => (StatusCode::FORBIDDEN, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

/// Extract authenticated user from request
pub struct AuthUser {
    pub claims: Claims,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Extract claims from extensions (set by middleware)
        let claims = parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or(AuthError::InvalidToken)?;

        Ok(AuthUser { claims })
    }
}

/// Middleware to validate JWT tokens
pub async fn auth_middleware(
    State(auth): axum::extract::State<Arc<AuthManager>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Get the Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(AuthError::InvalidToken)?;

    // Extract the token from "Bearer <token>"
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::InvalidToken)?;

    // Validate the token
    let claims = auth.validate_token(token)?;

    // Insert claims into request extensions
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_permissions() {
        assert!(Role::Admin.can_perform(Role::Admin));
        assert!(Role::Admin.can_perform(Role::Operator));
        assert!(Role::Admin.can_perform(Role::Viewer));

        assert!(!Role::Operator.can_perform(Role::Admin));
        assert!(Role::Operator.can_perform(Role::Operator));
        assert!(Role::Operator.can_perform(Role::Viewer));

        assert!(!Role::Viewer.can_perform(Role::Admin));
        assert!(!Role::Viewer.can_perform(Role::Operator));
        assert!(Role::Viewer.can_perform(Role::Viewer));
    }

    #[test]
    fn test_auth_manager_creation() {
        let auth = AuthManager::new("test-secret");
        let users = auth.users.read();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].username, "admin");
        assert_eq!(users[0].role, Role::Admin);
    }

    #[test]
    fn test_login_success() {
        let auth = AuthManager::new("test-secret");
        let result = auth.login(LoginRequest {
            username: "admin".to_string(),
            password: "admin".to_string(),
        });
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.token_type, "Bearer");
        assert_eq!(response.user.username, "admin");
        assert_eq!(response.user.role, Role::Admin);
    }

    #[test]
    fn test_login_invalid_credentials() {
        let auth = AuthManager::new("test-secret");
        let result = auth.login(LoginRequest {
            username: "admin".to_string(),
            password: "wrong".to_string(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_token_validation() {
        let auth = AuthManager::new("test-secret");
        let response = auth.login(LoginRequest {
            username: "admin".to_string(),
            password: "admin".to_string(),
        }).unwrap();

        let claims = auth.validate_token(&response.access_token);
        assert!(claims.is_ok());
        let claims = claims.unwrap();
        assert_eq!(claims.username, "admin");
        assert_eq!(claims.role, Role::Admin);
    }

    #[test]
    fn test_token_revocation() {
        let auth = AuthManager::new("test-secret");
        let response = auth.login(LoginRequest {
            username: "admin".to_string(),
            password: "admin".to_string(),
        }).unwrap();

        // Token should be valid initially
        assert!(auth.validate_token(&response.access_token).is_ok());

        // Revoke token
        auth.revoke_token(response.access_token.clone());

        // Token should now be invalid
        assert!(auth.validate_token(&response.access_token).is_err());
    }

    #[test]
    fn test_add_user() {
        let auth = AuthManager::new("test-secret");
        let result = auth.add_user(
            "operator".to_string(),
            "password123".to_string(),
            Role::Operator,
        );
        assert!(result.is_ok());

        let users = auth.users.read();
        assert_eq!(users.len(), 2);
        assert!(users.iter().any(|u| u.username == "operator" && u.role == Role::Operator));
    }

    #[test]
    fn test_add_duplicate_user() {
        let auth = AuthManager::new("test-secret");
        let result = auth.add_user(
            "admin".to_string(),
            "password123".to_string(),
            Role::Admin,
        );
        assert!(result.is_err());
    }
}
