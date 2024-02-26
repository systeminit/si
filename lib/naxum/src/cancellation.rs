use tokio_util::sync::CancellationToken;

pub async fn wait_on_cancelled(token: CancellationToken) {
    token.cancelled().await
}
