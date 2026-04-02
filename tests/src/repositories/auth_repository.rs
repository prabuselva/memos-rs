use crate::db::Database;
use crate::models::{AuthError, AuthResult, PasswordRecovery, Session, User};
use chrono::{Duration, Utc};
use uuid::Uuid;

const SESSION_DURATION_DAYS: i64 = 7;
const PASSWORD_RESET_DURATION_HOURS: i64 = 1;

pub struct AuthRepository;

impl Default for AuthRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthRepository {
    pub fn new() -> Self {
        Self
    }

    pub async fn register(
        &self,
        db: &Database,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> AuthResult<User> {
        let existing_user = db.get_user_by_username(username).await;
        if existing_user.is_ok() {
            return Err(AuthError::UsernameAlreadyExists);
        }

        let existing_email = db.get_user_by_email(email).await;
        if existing_email.is_ok() {
            return Err(AuthError::EmailAlreadyExists);
        }

        let user = User::new(
            username.to_string(),
            email.to_string(),
            password_hash.to_string(),
        );
        db.create_user(user.clone())
            .await
            .map_err(|e| AuthError::Database(format!("Failed to create user: {}", e)))?;

        Ok(user)
    }

    pub async fn verify_credentials(
        &self,
        db: &Database,
        username: &str,
        _password: &str,
    ) -> AuthResult<User> {
        let user = db
            .get_user_by_username(username)
            .await
            .map_err(|_| AuthError::InvalidCredentials)?;

        if !user.is_active {
            return Err(AuthError::Validation("Account is deactivated".to_string()));
        }

        Ok(user)
    }

    pub async fn create_session(&self, db: &Database, user_id: &str) -> AuthResult<Session> {
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::days(SESSION_DURATION_DAYS);

        let session = Session::new(user_id.to_string(), token, expires_at);
        db.create_session(session.clone())
            .await
            .map_err(|e| AuthError::Database(format!("Failed to create session: {}", e)))?;

        Ok(session)
    }

    pub async fn get_session(&self, db: &Database, session_token: &str) -> AuthResult<Session> {
        let session = db
            .get_session(session_token)
            .await
            .map_err(|_| AuthError::SessionNotFound)?;

        if !session.is_valid {
            return Err(AuthError::SessionNotFound);
        }

        if session.expires_at < Utc::now() {
            return Err(AuthError::TokenExpired);
        }

        Ok(session)
    }

    pub async fn get_user_profile_by_token(
        &self,
        db: &Database,
        session_token: &str,
    ) -> AuthResult<crate::models::UserProfile> {
        let session = self.get_session(db, session_token).await?;
        let user = db.get_user_by_id(&session.user_id).await?;
        Ok(crate::models::UserProfile::from(user))
    }

    pub async fn invalidate_session(&self, db: &Database, session_token: &str) -> AuthResult<()> {
        db.invalidate_session(session_token)
            .await
            .map_err(|e| AuthError::Database(format!("Failed to invalidate session: {}", e)))
    }

    pub async fn create_password_recovery(
        &self,
        db: &Database,
        user_id: &str,
    ) -> AuthResult<PasswordRecovery> {
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::hours(PASSWORD_RESET_DURATION_HOURS);

        let recovery = PasswordRecovery::new(user_id.to_string(), token, expires_at);
        db.create_password_recovery(recovery.clone())
            .await
            .map_err(|e| {
                AuthError::Database(format!("Failed to create password recovery: {}", e))
            })?;

        Ok(recovery)
    }

    pub async fn get_password_recovery(
        &self,
        db: &Database,
        token: &str,
    ) -> AuthResult<PasswordRecovery> {
        let recovery = db
            .get_password_recovery(token)
            .await
            .map_err(|_| AuthError::SessionNotFound)?;

        if recovery.is_used {
            return Err(AuthError::TokenInvalid);
        }

        if recovery.expires_at < Utc::now() {
            return Err(AuthError::TokenExpired);
        }

        Ok(recovery)
    }

    pub async fn mark_password_recovery_as_used(
        &self,
        db: &Database,
        token: &str,
    ) -> AuthResult<()> {
        db.mark_password_recovery_as_used(token)
            .await
            .map_err(|e| AuthError::Database(format!("Failed to mark recovery as used: {}", e)))
    }

    pub async fn get_user_by_id(&self, db: &Database, user_id: &str) -> AuthResult<User> {
        db.get_user_by_id(user_id)
            .await
            .map_err(|e| AuthError::Database(format!("Failed to get user: {}", e)))
    }

    pub async fn update_user_password(
        &self,
        db: &Database,
        user_id: &str,
        new_password_hash: &str,
    ) -> AuthResult<()> {
        let user = db
            .get_user_by_id(user_id)
            .await
            .map_err(|e| AuthError::Database(format!("Failed to get user: {}", e)))?;

        let updated_user = User {
            password_hash: new_password_hash.to_string(),
            updated_at: Utc::now(),
            ..user
        };

        db.update_user(updated_user)
            .await
            .map_err(|e| AuthError::Database(format!("Failed to update password: {}", e)))?;

        Ok(())
    }
}
