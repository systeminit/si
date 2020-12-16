CREATE TABLE edges
(
    id                       bigint PRIMARY KEY,
    si_id                    text UNIQUE,
    tail_vertex_node_si_id   text                     NOT NULL, -- Could be a ref to node id, but is probably an si id
    tail_vertex_object_si_id text                     NOT NULL, -- this is an si id
    tail_vertex_socket       text                     NOT NULL,
    tail_vertex_type_name    text                     NOT NULL,
    head_vertex_node_si_id   text                     NOT NULL, -- Could be a ref to node id, but is probably an si id
    head_vertex_object_si_id text                     NOT NULL, -- this is an si id
    head_vertex_socket       text                     NOT NULL,
    head_vertex_type_name    text                     NOT NULL,
    kind                     text                     NOT NULL,
    billing_account_id       bigint                   NOT NULL REFERENCES billing_accounts (id),
    organization_id          bigint                   NOT NULL REFERENCES organizations (id),
    workspace_id             bigint                   NOT NULL REFERENCES workspaces (id),
    epoch                    bigint                   NOT NULL,
    update_count             bigint                   NOT NULL,
    tenant_ids               text[]                   NOT NULL,
    obj                      jsonb                    NOT NULL,
    bidirectional            bool                     NOT NULL DEFAULT false,
    created_at               TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at               TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (workspace_id, tail_vertex_node_si_id, tail_vertex_object_si_id, tail_vertex_socket, tail_vertex_type_name,
            head_vertex_node_si_id, head_vertex_object_si_id, head_vertex_socket, head_vertex_type_name)
);

CREATE INDEX idx_edges_tenant_ids ON "edges" USING GIN ("tenant_ids");

CREATE OR REPLACE FUNCTION edge_create_v1(this_head_vertex_node_si_id text,
                                          this_head_vertex_object_si_id text,
                                          this_head_vertex_socket text,
                                          this_head_vertex_type_name text,
                                          this_tail_vertex_node_si_id text,
                                          this_tail_vertex_object_si_id text,
                                          this_tail_vertex_socket text,
                                          this_tail_vertex_type_name text,
                                          this_kind text,
                                          this_bidirectional bool,
                                          si_workspace_id text,
                                          this_epoch bigint,
                                          this_update_count bigint,
                                          OUT object jsonb) AS
$$
DECLARE
    this_id                 bigint;
    si_id                   text;
    this_workspace_id       bigint;
    this_organization_id    bigint;
    this_billing_account_id bigint;
    tenant_ids              text[];
    created_at              timestamp with time zone;
    updated_at              timestamp with time zone;
    si_storable             jsonb;
    this_head_vertex        jsonb;
    this_tail_vertex        jsonb;
BEGIN
    SELECT next_si_id_v1() INTO this_id;
    SELECT 'edge:' || this_id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT our_si_storable, our_organization_id, our_billing_account_id, our_workspace_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM si_storable_create_v1(si_id, si_workspace_id, created_at, updated_at, this_epoch, this_update_count);

    SELECT jsonb_build_object(
                   'nodeId', this_head_vertex_node_si_id,
                   'objectId', this_head_vertex_object_si_id,
                   'socket', this_head_vertex_socket,
                   'typeName', this_head_vertex_type_name
               )
    INTO this_head_vertex;

    SELECT jsonb_build_object(
                   'nodeId', this_tail_vertex_node_si_id,
                   'objectId', this_tail_vertex_object_si_id,
                   'socket', this_tail_vertex_socket,
                   'typeName', this_tail_vertex_type_name
               )
    INTO this_tail_vertex;

    SELECT jsonb_build_object(
                   'id', si_id,
                   'headVertex', this_head_vertex,
                   'tailVertex', this_tail_vertex,
                   'bidirectional', this_bidirectional,
                   'kind', this_kind,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO edges (id, si_id, tail_vertex_node_si_id, tail_vertex_object_si_id, tail_vertex_socket,
                       tail_vertex_type_name,
                       head_vertex_node_si_id, head_vertex_object_si_id, head_vertex_socket, head_vertex_type_name,
                       kind,
                       billing_account_id, organization_id, workspace_id, epoch, update_count, tenant_ids, obj,
                       bidirectional, created_at, updated_at)
    VALUES (this_id, si_id, this_tail_vertex_node_si_id, this_tail_vertex_object_si_id, this_tail_vertex_socket,
            this_tail_vertex_type_name, this_head_vertex_node_si_id, this_head_vertex_object_si_id,
            this_head_vertex_socket,
            this_head_vertex_type_name, this_kind, this_billing_account_id, this_organization_id, this_workspace_id,
            this_epoch, this_update_count, tenant_ids, object, this_bidirectional, created_at, updated_at);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION edge_get_v1(si_id text, OUT object jsonb) AS
$$
DECLARE
    this_id bigint;
BEGIN
    SELECT si_id_to_primary_key_v1(si_id) INTO this_id;
    SELECT w.obj INTO object FROM edges AS w WHERE id = this_id;
END
$$ LANGUAGE PLPGSQL STABLE;

CREATE OR REPLACE FUNCTION edge_delete_v1(si_id text) RETURNS void AS
$$
DECLARE
    this_id bigint;
BEGIN
    SELECT si_id_to_primary_key_v1(si_id) INTO this_id;
    DELETE FROM edges WHERE id = this_id;
END
$$ LANGUAGE PLPGSQL VOLATILE;