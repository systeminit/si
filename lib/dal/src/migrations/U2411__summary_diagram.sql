CREATE TABLE summary_diagram_components
(
    pk                       ident primary key                 default ident_create_v1(),
    id                       ident                    not null default ident_create_v1(),
    tenancy_workspace_pk     ident,
    visibility_change_set_pk ident                    NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at    timestamp with time zone,
    created_at               timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at               timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    component_id             ident                    NOT NULL,
    display_name             text                     NOT NULL,
    node_id                  ident                    NOT NULL,
    parent_node_id           ident,
    child_node_ids           jsonb,
    schema_name              text                     NOT NULL,
    schema_id                ident                    NOT NULL,
    schema_variant_id        ident                    NOT NULL,
    schema_variant_name      text                     NOT NULL,
    schema_category          text                     NOT NULL,
    position                 jsonb                    NOT NULL,
    size                     jsonb                    NOT NULL,
    color                    text                     NOT NULL,
    node_type                text                     NOT NULL,
    change_status            text                     NOT NULL,
    has_resource             boolean                  NOT NULL,
    created_info             jsonb                    NOT NULL,
    updated_info             jsonb                    NOT NULL,
    deleted_info             jsonb                    NOT NULL,
    sockets                  jsonb                    NOT NULL
);

SELECT standard_model_table_constraints_v1('summary_diagram_components');
INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('summary_diagram_components', 'model', 'summary_diagram_components', 'Summary Diagram Components');

CREATE OR REPLACE FUNCTION summary_diagram_component_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_id ident,
    this_schema_name text,
    this_schema_id ident,
    this_schema_variant_id ident,
    this_schema_variant_name text,
    this_schema_category text,
    this_sockets jsonb,
    this_node_id ident,
    this_display_name text,
    this_position jsonb,
    this_size jsonb,
    this_color text,
    this_node_type text,
    this_change_status text,
    this_has_resource boolean,
    this_created_info jsonb,
    this_updated_info jsonb,
    this_deleted_info jsonb,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           summary_diagram_components%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO summary_diagram_components (id, tenancy_workspace_pk, visibility_change_set_pk, visibility_deleted_at,
                                            component_id, display_name, node_id, schema_name,
                                            schema_id, schema_variant_id, schema_variant_name, schema_category,
                                            position, size, color, node_type, change_status, has_resource, created_info,
                                            updated_info, deleted_info, sockets, child_node_ids)
    VALUES (this_id, this_tenancy_record.tenancy_workspace_pk, this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at,
            this_id, this_display_name, this_node_id, this_schema_name, this_schema_id, this_schema_variant_id,
            this_schema_variant_name,
            this_schema_category, this_position, this_size, this_color, this_node_type, this_change_status,
            this_has_resource, this_created_info, this_updated_info, this_deleted_info,
            this_sockets, jsonb_build_array())
    RETURNING * INTO this_new_row;
END
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION summary_diagram_component_update_geometry_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_node_id ident,
    this_position jsonb,
    this_size jsonb,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           summary_diagram_components%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    UPDATE summary_diagram_components
    SET position=this_position,
        size=this_size
    WHERE node_id = this_node_id
      AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
      AND visibility_change_set_pk = this_visibility_record.visibility_change_set_pk
    RETURNING * INTO this_new_row;
END
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION summary_diagram_component_set_parent_node_id_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id ident,
    this_parent_node_id ident,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           summary_diagram_components%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    UPDATE summary_diagram_components
    SET parent_node_id=this_parent_node_id
    WHERE component_id = this_component_id
      AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
      AND visibility_change_set_pk = this_visibility_record.visibility_change_set_pk
    RETURNING * INTO this_new_row;
END
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION summary_diagram_component_unset_parent_node_id_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id ident,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           summary_diagram_components%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    UPDATE summary_diagram_components
    SET parent_node_id=NULL
    WHERE component_id = this_component_id
      AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
      AND visibility_change_set_pk = this_visibility_record.visibility_change_set_pk
    RETURNING * INTO this_new_row;
END
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION summary_diagram_component_update_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id ident,
    this_name text,
    this_color text,
    this_component_type text,
    this_has_resource bool,
    this_updated_info jsonb,
    this_deleted_at timestamp with time zone,
    this_deleted_info jsonb,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           summary_diagram_components%ROWTYPE;
    this_change_status     text;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);
    IF this_deleted_at IS NOT NULL then
        this_change_status := 'deleted';
    ELSE
        this_change_status := 'modified';
    END IF;

    -- First, we check to see if there is a row already for this change set. If there isn't, we copy the HEAD
    -- row with a few changes.
    IF NOT EXISTS (SELECT
                   FROM summary_diagram_components
                   WHERE component_id = this_component_id
                     AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
                     AND visibility_change_set_pk = this_visibility_record.visibility_change_set_pk) THEN
        INSERT INTO summary_diagram_components (id, tenancy_workspace_pk, visibility_change_set_pk,
                                                visibility_deleted_at, created_at, component_id,
                                                display_name, node_id, schema_name,
                                                schema_id, schema_variant_id,
                                                schema_variant_name, schema_category, position, size, color, node_type,
                                                change_status, has_resource, created_info, updated_info, deleted_info,
                                                sockets, parent_node_id, child_node_ids)
        SELECT id,
               tenancy_workspace_pk,
               this_visibility_record.visibility_change_set_pk AS visibility_change_set_pk,
               this_visibility_record.visibility_deleted_at    AS visibility_deleted_at,
               created_at,
               component_id,
               display_name,
               node_id,
               schema_name,
               schema_id,
               schema_variant_id,
               schema_variant_name,
               schema_category,
               position,
               size,
               color,
               node_type,
               this_change_status,
               has_resource,
               created_info,
               updated_info,
               deleted_info,
               sockets,
               parent_node_id,
               child_node_ids
        FROM summary_diagram_components
        WHERE id = this_component_id
          AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
          AND visibility_change_set_pk = '00000000000000000000000000';
    END IF;
    UPDATE summary_diagram_components
    SET display_name=this_name,
        color=this_color,
        node_type=this_component_type,
        has_resource=this_has_resource,
        updated_info=this_updated_info,
        visibility_deleted_at = this_deleted_at,
        deleted_info=this_deleted_info
    WHERE component_id = this_component_id
      AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
      AND visibility_change_set_pk = this_visibility_record.visibility_change_set_pk
    -- AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, summary_diagram_components)
    RETURNING * INTO this_new_row;
END
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION summary_diagram_component_change_status_trigger_v1() RETURNS trigger AS
$$
BEGIN
    IF NEW.change_status != 'deleted' THEN
        NEW.change_status := 'unmodified';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE PLPGSQL;

CREATE TRIGGER summary_diagram_component_change_status
    BEFORE INSERT OR UPDATE
    ON summary_diagram_components
    FOR EACH ROW
    WHEN (NEW.visibility_change_set_pk = '00000000000000000000000000')
EXECUTE FUNCTION summary_diagram_component_change_status_trigger_v1();

CREATE OR REPLACE FUNCTION summary_diagram_component_child_node_ids_trigger_v1() RETURNS trigger AS
$$
BEGIN
    IF NEW.parent_node_id IS NULL THEN
        IF OLD.parent_node_id IS NOT NULL THEN
            UPDATE summary_diagram_components
            SET child_node_ids = (SELECT COALESCE(json_agg(s1ni.node_id), json_build_array())
                                  FROM (SELECT DISTINCT ON (id) sdc1.node_id
                                        FROM summary_diagram_components as sdc1
                                        WHERE sdc1.tenancy_workspace_pk = NEW.tenancy_workspace_pk
                                          AND sdc1.parent_node_id = OLD.parent_node_id
                                          AND (sdc1.visibility_change_set_pk = NEW.visibility_change_set_pk
                                            AND (sdc1.visibility_deleted_at IS NULL OR
                                                 EXISTS (SELECT 1
                                                         FROM summary_diagram_components AS sdc2
                                                         WHERE sdc2.component_id = sdc1.component_id
                                                           AND sdc2.visibility_change_set_pk = ident_nil_v1()
                                                           AND sdc2.visibility_deleted_at IS NULL))
                                            )
                                        ORDER BY sdc1.id, sdc1.visibility_change_set_pk DESC,
                                                 sdc1.visibility_deleted_at DESC) as s1ni)
            WHERE node_id = OLD.parent_node_id;
        END IF;
    ELSE
        UPDATE summary_diagram_components
        SET child_node_ids = (SELECT COALESCE(json_agg(s1ni.node_id), json_build_array())
                              FROM (SELECT DISTINCT ON (id) sdc1.node_id
                                    FROM summary_diagram_components as sdc1
                                    WHERE sdc1.tenancy_workspace_pk = NEW.tenancy_workspace_pk
                                      AND sdc1.parent_node_id = NEW.parent_node_id
                                      AND (sdc1.visibility_change_set_pk = NEW.visibility_change_set_pk
                                        AND (sdc1.visibility_deleted_at IS NULL OR
                                             EXISTS (SELECT 1
                                                     FROM summary_diagram_components AS sdc2
                                                     WHERE sdc2.component_id = sdc1.component_id
                                                       AND sdc2.visibility_change_set_pk = ident_nil_v1()
                                                       AND sdc2.visibility_deleted_at IS NULL))
                                        )
                                    ORDER BY sdc1.id, sdc1.visibility_change_set_pk DESC,
                                             sdc1.visibility_deleted_at DESC) as s1ni)
        WHERE node_id = NEW.parent_node_id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE PLPGSQL;

CREATE TRIGGER summary_diagram_component_child_node_ids
    AFTER INSERT OR UPDATE
    ON summary_diagram_components
    FOR EACH ROW
EXECUTE FUNCTION summary_diagram_component_child_node_ids_trigger_v1();

CREATE TABLE summary_diagram_edges
(
    pk                       ident primary key                 default ident_create_v1(),
    id                       ident                    not null default ident_create_v1(),
    tenancy_workspace_pk     ident,
    visibility_change_set_pk ident                    NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at    timestamp with time zone,
    created_at               timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at               timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    edge_id                  ident                    NOT NULL,
    from_node_id             ident                    NOT NULL,
    from_socket_id           ident                    NOT NULL,
    to_node_id               ident                    NOT NULL,
    to_socket_id             ident                    NOT NULL,
    change_status            text                     NOT NULL,
    created_info             jsonb                    NOT NULL,
    deleted_info             jsonb
);

SELECT standard_model_table_constraints_v1('summary_diagram_edges');
INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('summary_diagram_edges', 'model', 'summary_diagram_edges', 'Summary Diagram Edges');

CREATE OR REPLACE FUNCTION summary_diagram_edge_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_id ident,
    this_from_node_id ident,
    this_from_socket_id ident,
    this_to_node_id ident,
    this_to_socket_id ident,
    this_created_info jsonb,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           summary_diagram_edges%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO summary_diagram_edges (id, tenancy_workspace_pk, visibility_change_set_pk, visibility_deleted_at,
                                       edge_id, from_node_id, from_socket_id, to_node_id,
                                       to_socket_id, change_status, created_info)
    VALUES (this_id, this_tenancy_record.tenancy_workspace_pk, this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at, this_id, this_from_node_id, this_from_socket_id,
            this_to_node_id, this_to_socket_id, 'added', this_created_info)
    RETURNING * INTO this_new_row;
END
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION summary_diagram_edge_change_status_trigger_v1() RETURNS trigger AS
$$
BEGIN
    IF NEW.change_status != 'deleted' THEN
        NEW.change_status := 'unmodified';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE PLPGSQL;

CREATE TRIGGER summary_diagram_edge_change_status
    BEFORE INSERT OR UPDATE
    ON summary_diagram_edges
    FOR EACH ROW
    WHEN (NEW.visibility_change_set_pk = '00000000000000000000000000')
EXECUTE FUNCTION summary_diagram_edge_change_status_trigger_v1();

CREATE OR REPLACE FUNCTION summary_diagram_edge_delete_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_id ident,
    this_visibility_deleted_at timestamp with time zone,
    this_deleted_info jsonb,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           summary_diagram_edges%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    -- First, we check to see if there is a row already for this change set. If there isn't, we copy the HEAD
    -- row with a few changes.
    IF NOT EXISTS (SELECT
                   FROM summary_diagram_edges
                   WHERE id = this_id
                     AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
                     AND visibility_change_set_pk = this_visibility_record.visibility_change_set_pk) THEN
        INSERT INTO summary_diagram_edges
        (id, tenancy_workspace_pk, visibility_change_set_pk, visibility_deleted_at, created_at, updated_at, edge_id,
         from_node_id, from_socket_id, to_node_id, to_socket_id, change_status, created_info, deleted_info)
        SELECT id,
               tenancy_workspace_pk,
               this_visibility_record.visibility_change_set_pk AS visibility_change_set_pk,
               this_visibility_deleted_at,
               created_at,
               updated_at,
               edge_id,
               from_node_id,
               from_socket_id,
               to_node_id,
               to_socket_id,
               change_status,
               created_info,
               deleted_info
        FROM summary_diagram_edges
        WHERE id = this_id
          AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
          AND visibility_change_set_pk = ident_nil_v1();
    END IF;

    UPDATE summary_diagram_edges
    SET visibility_deleted_at = this_visibility_deleted_at,
        deleted_info          = this_deleted_info,
        change_status         = 'deleted'
    WHERE id = this_id
      AND visibility_change_set_pk = this_visibility_record.visibility_change_set_pk
    RETURNING * INTO this_new_row;
END
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION edge_delete_updates_summaries_trigger_v1() RETURNS trigger AS
$$
DECLARE
    this_deleted_info json;
BEGIN
    IF NEW.visibility_deleted_at IS NOT NULL THEN
        IF NOT EXISTS (SELECT
                       FROM summary_diagram_edges
                       WHERE id = NEW.id
                         AND tenancy_workspace_pk = NEW.tenancy_workspace_pk
                         AND visibility_change_set_pk = NEW.visibility_change_set_pk) THEN
            INSERT INTO summary_diagram_edges
            (id, tenancy_workspace_pk, visibility_change_set_pk, visibility_deleted_at, created_at, updated_at, edge_id,
             from_node_id, from_socket_id, to_node_id, to_socket_id, change_status, created_info, deleted_info)
            SELECT id,
                   tenancy_workspace_pk,
                   NEW.visibility_change_set_pk,
                   NEW.visibility_deleted_at,
                   created_at,
                   updated_at,
                   edge_id,
                   from_node_id,
                   from_socket_id,
                   to_node_id,
                   to_socket_id,
                   change_status,
                   created_info,
                   deleted_info
            FROM summary_diagram_edges
            WHERE id = NEW.id
              AND tenancy_workspace_pk = NEW.tenancy_workspace_pk
              AND visibility_change_set_pk = ident_nil_v1();
        END IF;
        this_deleted_info := jsonb_build_object(
                'actor', jsonb_build_object(
                        'pk', COALESCE(NEW.deletion_user_pk, ident_nil_v1()),
                        'kind', 'system',
                        'email', 'system@systeminit.com',
                        'label', 'System Initiative'
                         ),
                'timestamp', NEW.visibility_deleted_at);
        UPDATE summary_diagram_edges
        SET visibility_deleted_at = NEW.visibility_deleted_at,
            change_status         = 'deleted',
            deleted_info          = this_deleted_info
        WHERE id = NEW.id
          AND tenancy_workspace_pk = NEW.tenancy_workspace_pk
          AND visibility_change_set_pk = NEW.visibility_change_set_pk;
    END IF;
    RETURN NEW;
END ;
$$ LANGUAGE PLPGSQL;

CREATE TRIGGER edge_delete_trigger
    AFTER INSERT OR UPDATE
    ON edges
    FOR EACH ROW
EXECUTE FUNCTION edge_delete_updates_summaries_trigger_v1();
