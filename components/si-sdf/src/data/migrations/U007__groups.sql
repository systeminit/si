CREATE TABLE groups
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    name               text                     NOT NULL,
    billing_account_id bigint                   NOT NULL REFERENCES billing_accounts(id),
    tenant_ids         text[]                   NOT NULL,
    obj                jsonb                    NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (name, billing_account_id)
);

CREATE INDEX idx_groups_tenant_ids ON "groups" USING GIN ("tenant_ids");

CREATE TABLE group_user_members
(
    group_id bigint NOT NULL REFERENCES groups (id),
    user_id  bigint NOT NULL REFERENCES users (id),
    PRIMARY KEY (group_id, user_id)
);

CREATE TABLE group_api_client_members
(
    group_id      bigint NOT NULL REFERENCES groups (id),
    api_client_id bigint NOT NULL REFERENCES api_clients (id),
    PRIMARY KEY (group_id, api_client_id)
);

CREATE TABLE group_capabilities
(
    group_id bigint NOT NULL REFERENCES groups (id),
    subject  text,
    action   text,
    PRIMARY KEY (group_id, subject, action)
);

/*
CREATE INDEX idx_groups_user_ids ON groups USING GIN(("obj"->'userIds'));
CREATE INDEX idx_groups_api_client_ids ON groups USING GIN(("obj"->'apiClientIds'));
CREATE INDEX idx_groups_capabilities ON groups USING GIN(("obj"->'capabilities'));
 */

CREATE OR REPLACE FUNCTION group_create_v1(name text,
                                           user_ids text[],
                                           api_client_ids text[],
                                           capabilities jsonb,
                                           billing_account_id text,
                                           OUT object jsonb) AS
$$
DECLARE
    id                    bigint;
    si_id                 text;
    billing_account_si_id bigint;
    tenant_ids            text[];
    created_at            timestamp with time zone;
    updated_at            timestamp with time zone;
    si_storable           jsonb;
    i_user_id             text;
    i_api_client_id       text;
    i_capability          jsonb;
BEGIN
    SELECT next_si_id_v1() INTO id;
    SELECT 'group:' || id INTO si_id;
    SELECT si_id_to_primary_key_v1(billing_account_id) INTO billing_account_si_id;
    SELECT ARRAY [billing_account_id, si_id] INTO tenant_ids;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;
    SELECT jsonb_build_object(
                   'typeName', 'group',
                   'tenantIds', tenant_ids,
                   'objectId', si_id,
                   'billingAccountId', billing_account_id,
                   'deleted', false,
                   'createdAt', created_at,
                   'updatedAt', updated_at
               )
    INTO si_storable;

    SELECT jsonb_build_object(
                   'id', si_id,
                   'name', name,
                   'userIds', user_ids,
                   'apiClientIds', api_client_ids,
                   'capabilities', capabilities,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO groups
    VALUES (id,
            si_id,
            name,
            billing_account_si_id,
            tenant_ids,
            object,
            created_at,
            updated_at);

    FOREACH i_user_id IN ARRAY user_ids
        LOOP
            INSERT INTO group_user_members VALUES (id, si_id_to_primary_key_v1(i_user_id));
        END LOOP;

    FOREACH i_api_client_id IN ARRAY api_client_ids
        LOOP
            INSERT INTO group_api_client_members VALUES (id, si_id_to_primary_key_v1(i_api_client_id));
        END LOOP;

    FOREACH i_capability IN ARRAY ARRAY [jsonb_array_elements(capabilities)]
        LOOP
            INSERT INTO group_capabilities VALUES (id, i_capability ->> 'subject', i_capability ->> 'action');
        END LOOP;
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION group_save_v1(input_group jsonb,
                                         OUT object jsonb) AS
$$
DECLARE
    this_current_group groups%rowtype;
    this_id            bigint;
    this_tenant_ids    text[];
    i_user_id          bigint;
    i_capability       jsonb;
    user_si_ids        text[];
    user_ids           bigint[];
    i_map_user_si_id   text;
    i_capability_array jsonb[];
    i_api_client_id          bigint;
    api_client_si_ids        text[];
    api_client_ids           bigint[];
    i_map_api_client_si_id   text;
BEGIN
    /* extract the id */
    SELECT si_id_to_primary_key_v1(input_group ->> 'id') INTO this_id;

    /* fetch the current user */
    SELECT * INTO this_current_group FROM groups WHERE id = this_id;
    IF NOT FOUND THEN
        RAISE WARNING 'group id % not found', this_id;
    END IF;

    IF si_id_to_primary_key_v1(input_group -> 'siStorable' ->> 'billingAccountId') !=
       this_current_group.billing_account_id THEN
        RAISE WARNING 'mutated billing account id; not allowed!';
    END IF;

    SELECT ARRAY(SELECT jsonb_array_elements_text(input_group -> 'siStorable' -> 'tenantIds')) INTO this_tenant_ids;

    UPDATE groups
    SET name       = input_group ->> 'name',
        tenant_ids = this_tenant_ids,
        obj        = input_group,
        updated_at = NOW()
    WHERE id = this_id
    RETURNING obj INTO object;

    /* We populate an array with the list of user_ids we are supposed to have, then
       we delete id that isn't in the set, and finally insert any that aren't present. */
    SELECT ARRAY[]::bigint[] INTO user_ids;

    SELECT ARRAY [jsonb_array_elements_text(input_group -> 'userIds')] INTO user_si_ids;
    FOR i_map_user_si_id IN SELECT * FROM jsonb_array_elements_text(input_group -> 'userIds')
        LOOP
            RAISE WARNING 'i have one %', i_map_user_si_id;
            SELECT array_append(user_ids, si_id_to_primary_key_v1(i_map_user_si_id)) INTO user_ids;
        END LOOP;

    RAISE WARNING 'this is users: %', user_ids;

    DELETE FROM group_user_members WHERE group_id = this_id AND ARRAY[user_id] NOT IN (user_ids);

    FOREACH i_user_id IN ARRAY user_ids
        LOOP
            INSERT INTO group_user_members VALUES (this_id, i_user_id) ON CONFLICT DO NOTHING;
        END LOOP;

    /* Same as above, but for api_client_ids */
    SELECT ARRAY[]::bigint[] INTO api_client_ids;

    SELECT ARRAY [jsonb_array_elements_text(input_group -> 'apiClientIds')] INTO api_client_si_ids;
    FOR i_map_api_client_si_id IN SELECT * FROM jsonb_array_elements_text(input_group -> 'apiClientIds')
        LOOP
            RAISE WARNING 'i have one %', i_map_api_client_si_id;
            SELECT array_append(api_client_ids, si_id_to_primary_key_v1(i_map_api_client_si_id)) INTO api_client_ids;
        END LOOP;

    RAISE WARNING 'this is api clients: %', api_client_ids;

    DELETE FROM group_api_client_members WHERE group_id = this_id AND ARRAY[api_client_id] NOT IN (api_client_ids);

    FOREACH i_api_client_id IN ARRAY api_client_ids
        LOOP
            INSERT INTO group_api_client_members VALUES (this_id, i_api_client_id) ON CONFLICT DO NOTHING;
        END LOOP;

    /* Nothing is poopier than a capability */
    DELETE
    FROM group_capabilities
    WHERE group_id = this_id
      AND (subject, action) NOT IN
          (SELECT poop ->> 'subject', poop ->> 'action'
           FROM jsonb_array_elements(input_group -> 'capabilities') as poop);

    SELECT ARRAY[jsonb_array_elements(input_group -> 'capabilities')] INTO i_capability_array;

    FOR i_capability IN SELECT * FROM jsonb_array_elements(input_group -> 'capabilities')
        LOOP
            INSERT INTO group_capabilities
            VALUES (this_id, i_capability ->> 'subject', i_capability ->> 'action') ON CONFLICT DO NOTHING;
        END LOOP;
END
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION group_get_v1(si_id text, OUT object jsonb) AS
$$
DECLARE
    this_id bigint;
BEGIN
    SELECT si_id_to_primary_key_v1(si_id) INTO this_id;
    SELECT w.obj INTO object FROM groups AS w WHERE id = this_id;
END
$$ LANGUAGE PLPGSQL STABLE;
