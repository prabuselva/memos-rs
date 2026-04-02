use crate::db::Database;
use crate::models::{AuthError, AuthResult, Session, User, UserProfile};
use crate::repositories::AuthRepository;
use crate::utils::auth_utils::{hash_password, verify_password};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AuthService {
    auth_repo: AuthRepository,
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthService {
    pub fn new() -> Self {
        Self {
            auth_repo: AuthRepository::new(),
        }
    }

    pub async fn register(
        &self,
        db: Arc<Mutex<Database>>,
        username: &str,
        email: &str,
        password: &str,
    ) -> AuthResult<(User, Session)> {
        let password_hash = hash_password(password)?;

        let db_guard = db.lock().await;
        let user = self
            .auth_repo
            .register(&db_guard, username, email, &password_hash)
            .await?;
        let session = self.auth_repo.create_session(&db_guard, &user.id).await?;

        Ok((user, session))
    }

    pub async fn login(
        &self,
        db: Arc<Mutex<Database>>,
        username: &str,
        password: &str,
    ) -> AuthResult<(User, Session)> {
        let db_guard = db.lock().await;

        let user = db_guard
            .get_user_by_username(username)
            .await
            .map_err(|_| AuthError::InvalidCredentials)?;

        if !user.is_active {
            return Err(AuthError::Validation("Account is deactivated".to_string()));
        }

        let is_valid = verify_password(password, &user.password_hash);
        if !is_valid {
            return Err(AuthError::InvalidCredentials);
        }

        let session = self.auth_repo.create_session(&db_guard, &user.id).await?;

        Ok((user, session))
    }

    pub async fn logout(&self, db: Arc<Mutex<Database>>, session_token: &str) -> AuthResult<()> {
        let db_guard = db.lock().await;
        self.auth_repo
            .invalidate_session(&db_guard, session_token)
            .await
    }

    pub async fn validate_session(
        &self,
        db: Arc<Mutex<Database>>,
        session_token: &str,
    ) -> AuthResult<UserProfile> {
        let db_guard = db.lock().await;
        let session = self.auth_repo.get_session(&db_guard, session_token).await?;
        let user = db_guard.get_user_by_id(&session.user_id).await?;

        Ok(UserProfile::from(user))
    }

    pub async fn request_password_reset(
        &self,
        db: Arc<Mutex<Database>>,
        email: &str,
    ) -> AuthResult<()> {
        let db_guard = db.lock().await;

        let user = match db_guard.get_user_by_username(email).await {
            Ok(u) => u,
            Err(_) => return Ok(()),
        };

        self.auth_repo
            .create_password_recovery(&db_guard, &user.id)
            .await?;

        Ok(())
    }

    pub async fn reset_password(
        &self,
        db: Arc<Mutex<Database>>,
        token: &str,
        password: &str,
    ) -> AuthResult<()> {
        let db_guard = db.lock().await;

        let recovery = self
            .auth_repo
            .get_password_recovery(&db_guard, token)
            .await?;

        let _password_hash = hash_password(password)?;
        let _user = db_guard.get_user_by_id(&recovery.user_id).await?;

        // In a real app, you'd update the user's password_hash here
        // For now, we just mark the recovery as used

        self.auth_repo
            .mark_password_recovery_as_used(&db_guard, token)
            .await?;

        Ok(())
    }

    pub async fn get_user_profile(
        &self,
        db: Arc<Mutex<Database>>,
        user_id: &str,
    ) -> AuthResult<UserProfile> {
        let db_guard = db.lock().await;
        let user = db_guard.get_user_by_id(user_id).await?;
        Ok(UserProfile::from(user))
    }

    pub async fn create_session(
        &self,
        db: &Database,
        user_id: &str,
    ) -> Result<Session, crate::models::AuthError> {
        let auth_repo = AuthRepository::new();
        auth_repo.create_session(db, user_id).await
    }

    pub async fn update_password(
        &self,
        db: Arc<Mutex<Database>>,
        user_id: &str,
        current_password: &str,
        new_password: &str,
    ) -> AuthResult<()> {
        let db_guard = db.lock().await;

        let user = db_guard
            .get_user_by_id(user_id)
            .await
            .map_err(|_| AuthError::UserNotFound(user_id.to_string()))?;

        let is_valid = verify_password(current_password, &user.password_hash);
        if !is_valid {
            return Err(AuthError::InvalidCredentials);
        }

        let new_password_hash = hash_password(new_password)?;
        self.auth_repo
            .update_user_password(&db_guard, user_id, &new_password_hash)
            .await?;

        Ok(())
    }
}
