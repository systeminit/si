DROP INDEX func_argument_name;
DROP INDEX intra_component_argument_with_two_internal_providers;
DROP INDEX intra_component_argument;
DROP INDEX inter_component_argument;

CREATE UNIQUE INDEX func_argument_name
    ON func_arguments (func_id,
                       name,
                       tenancy_workspace_pk,
                       visibility_change_set_pk,
                       COALESCE(visibility_deleted_at, '-infinity'));

CREATE UNIQUE INDEX intra_component_argument_with_two_internal_providers
    ON attribute_prototype_arguments (attribute_prototype_id,
                                      func_argument_id,
                                      internal_provider_id,
                                      head_component_id,
                                      tail_component_id,
                                      tenancy_workspace_pk,
                                      visibility_change_set_pk,
                                      COALESCE(visibility_deleted_at, '-infinity'))
    WHERE external_provider_id = ident_nil_v1();

CREATE UNIQUE INDEX intra_component_argument
    ON attribute_prototype_arguments (attribute_prototype_id,
                                      func_argument_id,
                                      internal_provider_id,
                                      tenancy_workspace_pk,
                                      visibility_change_set_pk,
                                      COALESCE(visibility_deleted_at, '-infinity'))
    WHERE head_component_id = ident_nil_v1()
          AND tail_component_id = ident_nil_v1();

CREATE UNIQUE INDEX inter_component_argument
    ON attribute_prototype_arguments (attribute_prototype_id,
                                      func_argument_id,
                                      external_provider_id,
                                      tail_component_id,
                                      head_component_id,
                                      tenancy_workspace_pk,
                                      visibility_change_set_pk,
                                      COALESCE(visibility_deleted_at, '-infinity'))
    WHERE internal_provider_id = ident_nil_v1();

