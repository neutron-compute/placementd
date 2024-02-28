///
/// Response objects
///

///
/// The v1 responses are used by the v1 API only
pub mod v1 {
    use placementd::db::TaskState;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    ///
    /// Response for a /runs/submit call that has completed
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    pub struct RunsSubmitted {
        pub ident: Uuid,
        pub state: TaskState,
    }
}
