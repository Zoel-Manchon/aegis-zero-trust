use crate::modules::auth::models::user_model::User;
use sqlx::PgPool;

pub struct UserRepository;

impl UserRepository {
    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password, user_role, created_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password, user_role, created_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create_user(
        pool: &PgPool,
        email: &str,
        hashed_password: &str,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, password)
            VALUES ($1, $2)
            RETURNING id, email, password, user_role, created_at
            "#,
        )
        .bind(email)
        .bind(hashed_password)
        .fetch_one(pool)
        .await
    }

    /// Update a user's password hash. Used by the password-reset flow.
    pub async fn update_password(
        pool: &PgPool,
        user_id: i64,
        new_password_hash: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE users
            SET password = $2
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(new_password_hash)
        .execute(pool)
        .await?;

        Ok(())
    }
}
