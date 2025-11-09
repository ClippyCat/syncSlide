use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum_login::{AuthUser, AuthnBackend, UserId};
use serde::{Deserialize, Serialize};
use sqlx::{self, SqlitePool};

/// Login form with username and password.
#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presentation {
    pub id: i64,
    pub user_id: i64,
    pub content: String,
    pub name: String,
}
impl Presentation {
    pub async fn new(user: &User, name: String, db: &SqlitePool) -> Result<Presentation, Error> {
        sqlx::query_as!(
            Presentation,
            "INSERT INTO presentation (user_id, name, content) VALUES (?, ?, ?)
            RETURNING *;",
            user.id,
            name,
            ""
        )
        .fetch_one(&*db)
        .await
        .map_err(Error::from)
    }
    pub async fn get_by_id(id: i64, db: &SqlitePool) -> Result<Option<Self>, Error> {
        sqlx::query_as!(Presentation, "SELECT * FROM presentation WHERE id = ?;", id)
            .fetch_optional(&*db)
            .await
            .map_err(Error::from)
    }
    pub async fn get_for_user(user: &User, db: &SqlitePool) -> Result<Vec<Self>, Error> {
        sqlx::query_as!(
            Presentation,
            "SELECT * FROM presentation WHERE user_id = ?;",
            user.id
        )
        .fetch_all(&*db)
        .await
        .map_err(Error::from)
    }
    pub async fn num_for_user(user: &User, db: &SqlitePool) -> Result<i64, Error> {
        sqlx::query_scalar!(
            "SELECT COUNT(id) as count FROM presentation WHERE user_id = ?;",
            user.id
        )
        .fetch_one(&*db)
        .await
        .map_err(Error::from)
    }
    pub async fn update_content(
        id: i64,
        new_content: String,
        db: &SqlitePool,
    ) -> Result<(), Error> {
        sqlx::query!(
            "UPDATE presentation
            SET content=?
            WHERE id=?",
            new_content,
            id
        )
        .execute(&*db)
        .await
        .map_err(Error::from)
        .map(|_| ())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: i64,
    name: String,
    email: String,
    password: String,
}
impl AuthUser for User {
    type Id = i64;
    fn id(&self) -> Self::Id {
        self.id
    }
    fn session_auth_hash(&self) -> &[u8] {
        self.password.as_bytes()
    }
}

#[derive(Clone)]
pub struct Backend {
    db: SqlitePool,
}
impl Backend {
    pub fn new(db: SqlitePool) -> Self {
        Backend { db }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Password(#[from] argon2::password_hash::Error),
}

impl AuthnBackend for Backend {
    type User = User;
    type Credentials = LoginForm;
    type Error = Error;
    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE name = ?;", creds.username)
            .fetch_optional(&self.db)
            .await?;
        let Some(user) = user else {
            return Ok(None);
        };
        let phash = PasswordHash::new(&user.password)?;
        if Argon2::default()
            .verify_password(creds.password.as_bytes(), &phash)
            .is_ok()
        {
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<User>, Error> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?;", user_id)
            .fetch_optional(&self.db)
            .await
            .map_err(Error::from)
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;
