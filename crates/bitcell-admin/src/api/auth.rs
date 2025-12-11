//! Authentication API endpoints

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{AppState, auth::{AuthUser, LoginRequest, RefreshRequest, Role}};

/// Login endpoint
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<crate::auth::AuthResponse>, crate::auth::AuthError> {
    let result = state.auth.login(req.clone());
    
    // Log authentication attempt
    match &result {
        Ok(response) => {
            state.audit.log_success(
                response.user.id.clone(),
                response.user.username.clone(),
                "login".to_string(),
                "auth".to_string(),
                None,
            );
        }
        Err(_) => {
            state.audit.log_failure(
                "unknown".to_string(),
                req.username.clone(),
                "login".to_string(),
                "auth".to_string(),
                "Invalid credentials".to_string(),
            );
        }
    }
    
    result.map(Json)
}

/// Refresh token endpoint
pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<crate::auth::AuthResponse>, crate::auth::AuthError> {
    let result = state.auth.refresh(req);
    
    // Log token refresh
    if let Ok(response) = &result {
        state.audit.log_success(
            response.user.id.clone(),
            response.user.username.clone(),
            "refresh_token".to_string(),
            "auth".to_string(),
            None,
        );
    }
    
    result.map(Json)
}

/// Logout endpoint (revokes token)
pub async fn logout(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
) -> Result<Json<LogoutResponse>, StatusCode> {
    // Extract token from header
    if let Some(auth_header) = req.headers().get(axum::http::header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                state.auth.revoke_token(token.to_string());
                
                state.audit.log_success(
                    user.claims.sub.clone(),
                    user.claims.username.clone(),
                    "logout".to_string(),
                    "auth".to_string(),
                    None,
                );
                
                return Ok(Json(LogoutResponse {
                    message: "Logged out successfully".to_string(),
                }));
            }
        }
    }
    
    Err(StatusCode::BAD_REQUEST)
}

#[derive(Serialize)]
pub struct LogoutResponse {
    pub message: String,
}

/// Create user endpoint (admin only)
#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: Role,
}

#[derive(Serialize)]
pub struct CreateUserResponse {
    pub id: String,
    pub username: String,
    pub role: Role,
}

pub async fn create_user(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<CreateUserResponse>, crate::auth::AuthError> {
    // Only admin can create users
    if user.claims.role != Role::Admin {
        state.audit.log_failure(
            user.claims.sub.clone(),
            user.claims.username.clone(),
            "create_user".to_string(),
            req.username.clone(),
            "Insufficient permissions".to_string(),
        );
        return Err(crate::auth::AuthError::InsufficientPermissions);
    }
    
    let result = state.auth.add_user(req.username.clone(), req.password, req.role);
    
    match &result {
        Ok(new_user) => {
            state.audit.log_success(
                user.claims.sub.clone(),
                user.claims.username.clone(),
                "create_user".to_string(),
                new_user.username.clone(),
                Some(format!("Created user with role: {:?}", new_user.role)),
            );
            
            Ok(Json(CreateUserResponse {
                id: new_user.id.clone(),
                username: new_user.username.clone(),
                role: new_user.role,
            }))
        }
        Err(e) => {
            state.audit.log_failure(
                user.claims.sub.clone(),
                user.claims.username.clone(),
                "create_user".to_string(),
                req.username,
                e.to_string(),
            );
            Err(e.clone())
        }
    }
}

/// Get audit logs endpoint (admin and operator can view)
#[derive(Deserialize)]
pub struct AuditLogsQuery {
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    100
}

#[derive(Serialize)]
pub struct AuditLogsResponse {
    pub logs: Vec<crate::audit::AuditLogEntry>,
    pub total: usize,
}

pub async fn get_audit_logs(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    axum::extract::Query(query): axum::extract::Query<AuditLogsQuery>,
) -> Result<Json<AuditLogsResponse>, StatusCode> {
    // Only admin and operator can view audit logs
    if !matches!(user.claims.role, Role::Admin | Role::Operator) {
        return Err(StatusCode::FORBIDDEN);
    }
    
    let all_logs = state.audit.get_logs();
    let total = all_logs.len();
    let logs = state.audit.get_recent_logs(query.limit);
    
    state.audit.log_success(
        user.claims.sub.clone(),
        user.claims.username.clone(),
        "view_audit_logs".to_string(),
        "audit".to_string(),
        Some(format!("Retrieved {} logs", logs.len())),
    );
    
    Ok(Json(AuditLogsResponse { logs, total }))
}
