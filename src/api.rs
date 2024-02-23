///
/// The API module contains all the necessary routes for the API

/// The v1 API
pub mod v1 {
    use crate::types::databricks::*;
    use crate::types::responses::v1::*;
    use crate::types::*;

    use tide::{Body, Result};
    use tracing::log::*;

    /// Return the API routes for nesting
    pub fn routes(state: ServerState) -> Result<Server> {
        let mut app = tide::with_state(state);
        app.at("/runs/submit").post(runs_submit);
        debug!("Registered API routes: {app:?}");
        Ok(app)
    }

    ///
    /// POST /runs/submit
    pub async fn runs_submit(mut req: Request) -> Result<Body> {
        //let request: RunsSubmitRequest = req.body_json().await?;
        let request: serde_json::Value = req.body_json().await?;
        println!("Recevied: {request:?}");
        let task = crate::dal::Task::default();
        let mut pool = req.state().pool.clone();
        let ident = task.save(&mut pool).await?;

        let response = RunsSubmitted {
            ident,
            state: task.state,
        };
        Ok(Body::from_json(&response)?)
    }

    #[cfg(feature = "integration")]
    #[cfg(test)]
    mod integration_tests {
        use super::*;
        use tide_testing::TideTestingExt;
        use uuid::Uuid;

        /// Return a constructed test [tide::Server] of the API
        async fn test_api() -> Server {
            let state = ServerState::from_env().await;
            super::routes(state).expect("Failed to get routes")
        }

        #[async_std::test]
        async fn test_runs_submit() -> Result<()> {
            let app = test_api().await;

            let payload =
                String::from_utf8_lossy(&std::fs::read("tests/example-runs-submit.json")?).into();

            let response: RunsSubmitted = app
                .post("/runs/submit")
                .body(Body::from_string(payload))
                .content_type("application/json")
                .recv_json()
                .await?;

            assert_eq!(response.state, crate::dal::TaskState::Planned);
            assert_ne!(response.ident, Uuid::new_v4());
            Ok(())
        }
    }
}
