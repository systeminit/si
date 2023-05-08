CREATE UNIQUE INDEX prop_unique_path
    ON props (path,
              schema_variant_id,
              tenancy_workspace_pk,
              visibility_change_set_pk);

ALTER TABLE props ADD COLUMN refers_to_prop_id ident;

