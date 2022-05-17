CREATE TABLE external_providers
(
    pk bigserial PRIMARY KEY,
    id bigserial NOT NULL,
    tenancy_universal bool NOT NULL,
    tenancy_billing_account_ids bigint[],
    tenancy_organization_ids bigint[],
    tenancy_workspace_ids bigint[],
    visibility_change_set_pk bigint,
    visibility_edit_session_pk bigint,
    visibility_deleted_at timestamp with time zone,
    attribute_context_prop_id bigint,
    attribute_context_schema_id bigint,
    attribute_context_schema_variant_id bigint,
    attribute_context_component_id bigint,
    attribute_context_system_id bigint,
    created_at timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at timestamp with time zone NOT NULL DEFAULT NOW(),
    prop_id bigint,
    schema_id bigint,
    schema_variant_id bigint,
    name text,
    type_definition text,
    attribute_prototype_id bigint
);
SELECT standard_model_table_constraints_v1('external_providers');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
    VALUES ('external_providers', 'model', 'external_provider', 'Output Provider');

CREATE OR REPLACE FUNCTION external_provider_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_prop_id bigint,
    this_schema_id bigint,
    this_schema_variant_id bigint,
    this_name text,
    this_type_definition text,
    this_attribute_prototype_id bigint,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record           tenancy_record_v1;
    this_visibility_record        visibility_record_v1;
    this_new_row                  external_providers%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO external_providers (tenancy_universal,
                               tenancy_billing_account_ids,
                               tenancy_organization_ids,
                               tenancy_workspace_ids,
                               visibility_change_set_pk,
                               visibility_edit_session_pk,
                               visibility_deleted_at,
                               prop_id,
                               schema_id,
                               schema_variant_id,
                               name,
                               type_definition,
                               attribute_prototype_id)
    VALUES (this_tenancy_record.tenancy_universal,
            this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids,
            this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_edit_session_pk,
            this_visibility_record.visibility_deleted_at,
            this_prop_id,
            this_schema_id,
            this_schema_variant_id,
            this_name,
            this_type_definition,
            this_attribute_prototype_id)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;