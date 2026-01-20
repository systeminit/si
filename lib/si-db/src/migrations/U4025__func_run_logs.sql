CREATE TABLE func_run_logs
(
    key               text                     NOT NULL PRIMARY KEY,
    sort_key          text                     NOT NULL,
    created_at        timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at        timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),

    workspace_id      text                     NOT NULL,
    change_set_id     text                     NOT NULL,

    func_run_id       text                     NOT NULL UNIQUE,
    value             bytea                    NOT NULL,
    serialization_lib text                     NOT NULL DEFAULT 'postcard'
);

CREATE INDEX IF NOT EXISTS func_run_logs_sort_key ON func_run_logs (sort_key);
CREATE INDEX IF NOT EXISTS func_run_log_by_func_run_id ON func_run_logs (func_run_id);
