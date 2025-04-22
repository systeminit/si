use si_data_nats::{
    Subject,
    jetstream::Context,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub(crate) context: Context,
    pub(crate) destination_subject: Subject,
    pub(crate) self_shutdown_token: CancellationToken,
}

impl AppState {
    pub(crate) fn new(
        context: Context,
        destination_subject: Subject,
        self_shutdown_token: CancellationToken,
    ) -> Self {
        Self {
            context,
            destination_subject,
            self_shutdown_token,
        }
    }
}
