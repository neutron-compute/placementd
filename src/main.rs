use std::env::*;
use tracing::log::*;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    info!("Starting placementd");
    let app = tide::new();
    let bind_to = var("BIND_TO").unwrap_or("0.0.0.0:8080".into());
    info!("Starting the HTTP handler on {bind_to}");
    app.listen(bind_to).await?;

    Ok(())
}

/// The data access objects for placemerntd
mod dao {
    use chrono::prelude::*;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    ///
    /// The state of a task in the work scheduling system
    ///
    #[derive(sqlx::Type, Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
    #[sqlx(type_name = "placementd_state", rename_all = "lowercase")]
    enum TaskState {
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

    ///
    /// A Task is the core of `placementd` and represents work to be done.
    ///
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    struct Task {
        /// Globally unique identifier for this [Task]
        ident: Uuid,
        /// The current [TaskState]
        state: TaskState,
        /// The UTC datetime for when the [Task] was created
        created: DateTime<Utc>,
        /// UTC time when the [Task] last had an update of any kind.
        updated: Option<DateTime<Utc>>,
        /// The time when the [Task] entered the `Completed` [TaskState]
        completed: Option<DateTime<Utc>>,
        // User-defined tags for the [Task]
        tags: Option<serde_json::Value>,
    }

    ///
    /// A Manifest is the raw user input which creates a [Task]
    ///
    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct Manifest {
        /// The [Task] identifier for this [Manifest]
        ident: Uuid,
        manifest: serde_json::Value,
        /// The UTC datetime for when the [Manifest] was created
        created: DateTime<Utc>,
    }

    impl Task {
        async fn save(&self, pool: &mut sqlx::PgPool) -> Result<Uuid, sqlx::Error> {
            let mut tx = pool.begin().await?;
            let ident = Uuid::new_v4();
            let _ = sqlx::query!(r#"INSERT INTO tasks (ident) VALUES ($1)"#, ident)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            Ok(ident)
        }

        async fn lookup(ident: &Uuid, pool: &sqlx::PgPool) -> Result<Self, sqlx::Error> {
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
        }
    }

    #[cfg(feature = "integration")]
    #[cfg(test)]
    mod integration_tests {
        use super::*;

        #[cfg(feature = "postgres")]
        async fn test_pool() -> sqlx::PgPool {
            // These hard-coded credentials are mirrored in develop/postgres.yml
            let database_url = std::env::var("DATABASE_URL")
                .unwrap_or("postgres://placementd:VerySecure!@127.0.0.1:5432".into());
            sqlx::PgPool::connect(&database_url)
                .await
                .expect("Failed to connectto {database_url}")
        }

        #[async_std::test]
        async fn test_default_integration() {
            let task = Task::default();
            let mut pool = test_pool().await;
            let ident = task
                .save(&mut pool)
                .await
                .expect("Saving the test task should not fail");

            let task = Task::lookup(&ident, &pool)
                .await
                .expect("Failed to look up");
            assert_eq!(task.ident, ident);
        }
    }
}

#[cfg(test)]
mod tests {}
