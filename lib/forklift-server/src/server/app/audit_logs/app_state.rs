use audit_database::AuditDatabaseContext;

// NOTE(nick): we need an app state for all naxum apps at the time of writing, even if they are unused.
#[derive(Debug, Clone)]
pub(crate) struct AppState {
    context: AuditDatabaseContext,
    using_prefix: bool,
}

impl AppState {
    pub(crate) fn new(context: AuditDatabaseContext, using_prefix: bool) -> Self {
        Self {
            context,
            using_prefix,
        }
    }

    pub(crate) fn context(&self) -> &AuditDatabaseContext {
        &self.context
    }

    pub(crate) fn using_prefix(&self) -> bool {
        self.using_prefix
    }
}
