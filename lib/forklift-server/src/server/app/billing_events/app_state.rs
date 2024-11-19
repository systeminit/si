use data_warehouse_stream_client::DataWarehouseStreamClient;

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub(crate) data_warehouse_stream_client: DataWarehouseStreamClient,
}

impl AppState {
    pub(crate) fn new(data_warehouse_stream_client: DataWarehouseStreamClient) -> Self {
        Self {
            data_warehouse_stream_client,
        }
    }
}

// NOTE(nick,fletcher): we need an app state for all naxum apps at the time of writing, even if they are unused.
#[derive(Debug, Clone)]
pub(crate) struct NoopAppState {}

impl NoopAppState {
    pub(crate) fn new() -> Self {
        Self {}
    }
}
