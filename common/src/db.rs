use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Convenience type alias since
type Transaction<'a> = sqlx::Transaction<'a, sqlx::Postgres>;

///
/// Bootstrap the [sqlx::PgPool] connection using the `DATABASE_URL` environment variable or
/// fall back to the default development credentials
///
pub async fn bootstrap() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or("postgres://placementd:VerySecure!@127.0.0.1:5432".into());
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connectto {database_url}")
}

/// A raw submitted run for placementd
///
/// This entity is related to a [Task] but captured the raw submitted request from the end
/// users. This allows for more debuggability and flexibility when (re-)generating resources
/// launched into the compute provisioners
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubmittedRun {
    ident: Uuid,
    raw: serde_json::Value,
    task: Uuid,
    tags: Option<serde_json::Value>,
    /// The UTC datetime for when the [Task] was created
    created: DateTime<Utc>,
    /// UTC time when the [Task] last had an update of any kind.
    updated: Option<DateTime<Utc>>,
    /// The time when the [Task] entered the `Completed` [TaskState]
    completed: Option<DateTime<Utc>>,
}

impl SubmittedRun {
    ///
    /// Create and return a [SubmittedRun] which references the given [Task]
    pub async fn create(
        tx: &mut Transaction<'_>,
        task: &Task,
        raw: serde_json::Value,
    ) -> Result<Self, sqlx::Error> {
        let run = sqlx::query_as!(
            Self,
            r#"
                INSERT INTO submitted_runs (ident, task, raw)
                            VALUES ($1, $2, $3)
                RETURNING
                    ident,
                    raw,
                    task,
                    COALESCE(hstore_to_json(tags), '{}'::json) AS tags,
                    created,
                    updated,
                    completed
            "#,
            Uuid::new_v4(),
            &task.ident,
            raw
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(run)
    }
}

///
/// The state of a task in the work scheduling system
///
#[derive(sqlx::Type, Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum TaskState {
    #[default]
    /// Planned is a state where placementd has received the request
    Planned,
    /// Provisioning is when infrastructure is being provisioend for this task
    Provisioning,
    /// The task has been submitted to the infrastructure and is considered running
    Running,
    /// The task has completed in the infrastructure but resources are not yet cleaned up
    Completed,
    /// The task has completed and resources have been cleaned up
    Finalized,
}

/// Conversion for taking [TaskState] to a string for storage in PostgreSQL
///
/// The `strum` crate could probably make this simpler, but for 5 variants it is not worth the
/// depende3ncy surface area
impl Into<String> for TaskState {
    fn into(self) -> String {
        match self {
            Self::Planned => "planned",
            Self::Provisioning => "provisioning",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Finalized => "finalized",
        }
        .into()
    }
}

///
/// A Task is the core of `placementd` and represents work to be done.
///
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Task {
    /// Globally unique identifier for this [Task]
    pub ident: Uuid,
    /// The current [TaskState]
    pub state: TaskState,
    // User-defined tags for the [Task]
    tags: Option<serde_json::Value>,
    /// The UTC datetime for when the [Task] was created
    created: DateTime<Utc>,
    /// UTC time when the [Task] last had an update of any kind.
    updated: Option<DateTime<Utc>>,
    /// The time when the [Task] entered the `Completed` [TaskState]
    completed: Option<DateTime<Utc>>,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            ident: Uuid::new_v4(),
            state: TaskState::default(),
            tags: None,
            created: Utc::now(),
            updated: None,
            completed: None,
        }
    }
}

impl Task {
    /// Create a new Task in the database
    pub async fn create(tx: &mut Transaction<'_>) -> Result<Self, sqlx::Error> {
        let task = Self::default();

        let _ = sqlx::query!(r#"INSERT INTO tasks (ident) VALUES ($1)"#, task.ident)
            .execute(&mut **tx)
            .await?;

        Ok(task)
    }

    pub async fn lock(
        tx: &mut Transaction<'_>,
        ident: &Uuid,
        state: Option<TaskState>,
    ) -> Result<Self, sqlx::Error> {
        let state: String = state.unwrap_or(TaskState::Planned).into();

        sqlx::query_as!(
            Self,
            r#"
            SELECT ident,
                    created,
                    updated,
                    completed,
                    state AS "state!: TaskState",
                    COALESCE(hstore_to_json(tags), '{}'::json) AS tags
                    FROM tasks
                    WHERE
                        ident = $1
                        AND state = $2
                -- Lock the row for our transaction
                FOR UPDATE SKIP LOCKED
                        "#,
            ident,
            state,
        )
        .fetch_one(&mut **tx)
        .await
    }

    pub async fn lookup(ident: &Uuid, pool: &sqlx::PgPool) -> Result<Self, sqlx::Error> {
        // sqlx doesn't directly support HStore, but can be coalesced to JSON, inspired by:
        // <https://stackoverflow.com/a/76855805>
        sqlx::query_as!(
            Self,
            r#"SELECT ident,
                created,
                updated,
                completed,
                COALESCE(hstore_to_json(tags), '{}'::json) AS tags,
                state AS "state!: TaskState"
        FROM tasks WHERE ident = $1"#,
            ident
        )
        .fetch_one(pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_task() {
        let task = Task::default();
        assert_eq!(
            task.state,
            TaskState::Planned,
            "The tasks should be Planning by default"
        );
        assert!(task.completed.is_none());
        assert!(task.updated.is_none());
    }
}

#[cfg(feature = "integration")]
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[async_std::test]
    async fn test_task_create() {
        let pool = bootstrap().await;
        let mut tx = pool.begin().await.expect("Failed to start transaction");
        let _task = Task::create(&mut tx).await;
        tx.commit().await.expect("Failed to commit transaction");
    }

    #[async_std::test]
    async fn test_task_lock() {
        let pool = bootstrap().await;

        let mut tx = pool.begin().await.expect("Failed to start transaction");
        let original_task = Task::create(&mut tx).await.expect("Failed to create task");
        tx.commit().await.expect("Failed to commit transaction");

        let mut tx = pool.begin().await.expect("Failed to start transaction");
        let task = Task::lock(&mut tx, &original_task.ident, None)
            .await
            .expect("Failed to lock row");
        assert_eq!(task.ident, original_task.ident);
        assert_eq!(task.state, TaskState::Planned);

        tx.commit().await.expect("Failed to commit transaction");
    }

    #[async_std::test]
    async fn test_task_lock_conflict() {
        use async_std::prelude::FutureExt;

        let pool = bootstrap().await;

        let mut tx = pool.begin().await.expect("Failed to start transaction");
        let original_task = Task::create(&mut tx).await.expect("Failed to create task");
        tx.commit().await.expect("Failed to commit transaction");

        let mut tx = pool.begin().await.expect("Failed to start transaction");
        let locker = async_std::task::spawn(async move {
            let task = Task::lock(&mut tx, &original_task.ident, None)
                .await
                .expect("Failed to lock row");
            assert_eq!(task.ident, original_task.ident);
            assert_eq!(task.state, TaskState::Planned);

            // wait!
            async_std::task::sleep(std::time::Duration::from_secs(1)).await;

            tx.commit().await.expect("Failed to commit transaction");
        });

        let checker = async_std::task::spawn(async move {
            let mut tx = pool.begin().await.expect("Failed to start transaction");
            match Task::lock(&mut tx, &original_task.ident, None).await {
                Ok(_) => assert!(
                    false,
                    "Should not have been able to return a non-existent record"
                ),
                Err(sqlx::Error::RowNotFound) => {} // expected
                Err(others) => assert!(false, "Got an unexpected error: {others:?}"),
            }
            tx.commit().await.expect("Failed to commit transaction");
        });

        locker.join(checker).await;
    }

    #[async_std::test]
    async fn test_task_lock_unavailable() {
        let pool = bootstrap().await;

        let mut tx = pool.begin().await.expect("Failed to start transaction");
        let fake = Uuid::new_v4();
        match Task::lock(&mut tx, &fake, None).await {
            Ok(_) => assert!(
                false,
                "Should not have been able to return a non-existent record"
            ),
            Err(sqlx::Error::RowNotFound) => {} // expected
            Err(others) => assert!(false, "Got an unexpected error: {others:?}"),
        }
        tx.commit().await.expect("Failed to commit transaction");
    }
}
