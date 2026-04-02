pub mod auth;
pub mod auth_dto;
pub mod note;

pub use auth::{AuthError, AuthResult, PasswordRecovery, Session, User, UserProfile};
pub use auth_dto::{
    LoginRequest, LoginResponse, PasswordResetConfirm, PasswordResetRequest, RegisterRequest,
    RegisterResponse, UpdateProfileRequest,
};
pub use note::{Note, NoteError, NoteResult, Notebook, Tag};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookNode {
    pub notebook: Notebook,
    pub children: Vec<NotebookNode>,
    pub notes: Vec<Note>,
}

impl NotebookNode {
    pub fn new(notebook: Notebook) -> Self {
        Self {
            notebook,
            children: Vec::new(),
            notes: Vec::new(),
        }
    }
}
