-- Limit values of action_prototypes.kind to a known set of variants. Is this required? No! But such a constraint
-- might be useful elsewhere
ALTER TABLE action_prototypes DROP CONSTRAINT valid_kind_check;

ALTER TABLE action_prototypes
    ADD CONSTRAINT valid_kind_check CHECK (kind IN ('create', 'refresh', 'other', 'destroy'));
