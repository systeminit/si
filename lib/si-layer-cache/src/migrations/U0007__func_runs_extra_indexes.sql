DROP INDEX IF EXISTS by_attribute_value_id;
CREATE INDEX IF NOT EXISTS func_runs_by_attribute_value_id ON func_runs (attribute_value_id, workspace_id, updated_at DESC);
CREATE INDEX IF NOT EXISTS func_runs_function_kind_and_workspace_id ON func_runs (function_kind, workspace_id, updated_at DESC);
CREATE INDEX IF NOT EXISTS func_runs_action_id_and_workspace_id ON func_runs (action_id, workspace_id, updated_at DESC);
