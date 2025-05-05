pub const WORKSPACE_GET_BY_PK: &str = include_str!("queries/workspace/get_by_pk.sql");
pub const WORKSPACE_LIST_FOR_USER: &str = include_str!("queries/workspace/list_for_user.sql");
pub const WORKSPACE_LIST_ALL: &str = include_str!("queries/workspace/list_all.sql");
pub const SEARCH_WORKSPACES_BY_ULID: &str = include_str!("queries/workspace/search_ulid.sql");
pub const SEARCH_WORKSPACES_BY_SNAPSHOT_ADDRESS: &str =
    include_str!("queries/workspace/search_snapshot_address.sql");
pub const SEARCH_WORKSPACES_USER_NAME_EMAIL: &str =
    include_str!("queries/workspace/search_user_name_email.sql");
