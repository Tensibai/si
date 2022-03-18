mod view;

use veritech::EncryptionKey;
pub use view::{ComponentView, ComponentViewError};

use async_recursion::async_recursion;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute_resolver::{AttributeResolverContext, UNSET_ID_VALUE};
use crate::code_generation_resolver::CodeGenerationResolverContext;
use crate::deculture::attribute::value::AttributeValueError;
use crate::edit_field::{
    value_and_visibility_diff_json_option, widget::prelude::*, EditField, EditFieldAble,
    EditFieldBaggage, EditFieldBaggageComponentProp, EditFieldError, EditFieldObjectKind,
    EditFields,
};
use crate::func::backend::integer::FuncBackendIntegerArgs;
use crate::func::backend::map::FuncBackendMapArgs;
use crate::func::backend::validation::{FuncBackendValidateStringValueArgs, ValidationError};
use crate::func::backend::{
    js_attribute::FuncBackendJsAttributeArgs, js_code_generation::FuncBackendJsCodeGenerationArgs,
    js_qualification::FuncBackendJsQualificationArgs, js_resource::FuncBackendJsResourceSyncArgs,
    string::FuncBackendStringArgs,
};
use crate::func::binding::{FuncBinding, FuncBindingError};
use crate::func::binding_return_value::FuncBindingReturnValue;
use crate::node::NodeKind;
use crate::qualification::QualificationView;
use crate::qualification_resolver::QualificationResolverContext;
use crate::resource_resolver::ResourceResolverContext;
use crate::schema::variant::{SchemaVariantError, SchemaVariantId};
use crate::schema::SchemaVariant;
use crate::validation_resolver::ValidationResolverContext;
use crate::ws_event::{WsEvent, WsEventError};

use crate::func::backend::array::FuncBackendArrayArgs;
use crate::func::backend::boolean::FuncBackendBooleanArgs;
use crate::func::backend::prop_object::FuncBackendPropObjectArgs;
use crate::{
    impl_standard_model, pk, qualification::QualificationError, standard_model,
    standard_model_accessor, standard_model_belongs_to, standard_model_has_many, AttributeResolver,
    AttributeResolverError, AttributeResolverId, CodeGenerationPrototype,
    CodeGenerationPrototypeError, CodeGenerationResolver, CodeGenerationResolverError, Edge,
    EdgeError, Func, FuncBackendKind, HistoryActor, HistoryEventError, LabelEntry, LabelList, Node,
    NodeError, OrganizationError, Prop, PropError, PropId, PropKind, QualificationPrototype,
    QualificationPrototypeError, QualificationResolver, QualificationResolverError, ReadTenancy,
    ReadTenancyError, Resource, ResourceError, ResourcePrototype, ResourcePrototypeError,
    ResourceResolver, ResourceResolverError, ResourceView, Schema, SchemaError, SchemaId, Secret,
    StandardModel, StandardModelError, System, SystemId, Tenancy, Timestamp, ValidationPrototype,
    ValidationPrototypeError, ValidationResolver, ValidationResolverError, Visibility,
    WorkspaceError, WriteTenancy,
};

#[derive(Error, Debug)]
pub enum ComponentError {
    #[error("AttributeValue error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("edit field error: {0}")]
    EditField(#[from] EditFieldError),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("qualification prototype error: {0}")]
    QualificationPrototype(#[from] QualificationPrototypeError),
    #[error("qualification resolver error: {0}")]
    QualificationResolver(#[from] QualificationResolverError),
    #[error("resource prototype error: {0}")]
    ResourcePrototype(#[from] ResourcePrototypeError),
    #[error("resource resolver error: {0}")]
    ResourceResolver(#[from] ResourceResolverError),
    #[error("code generation prototype error: {0}")]
    CodeGenerationPrototype(#[from] CodeGenerationPrototypeError),
    #[error("code generation resolver error: {0}")]
    CodeGenerationResolver(#[from] CodeGenerationResolverError),
    #[error("unable to find code generated")]
    CodeGeneratedNotFound,
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("node error: {0}")]
    NodeError(#[from] NodeError),
    #[error("component not found: {0}")]
    NotFound(ComponentId),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("resource not found for component ({0}) in system ({1})")]
    ResourceNotFound(ComponentId, SystemId),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("unable to find system")]
    SystemNotFound,
    #[error("attribute resolver error: {0}")]
    AttributeResolver(#[from] AttributeResolverError),
    #[error("missing attribute resolver: {0}")]
    MissingAttributeResolver(AttributeResolverId),
    #[error("missing parent attribute resolver for: {0}")]
    MissingParentAttributeResolver(AttributeResolverId),
    #[error("missing a prop in attribute update: {0} not found")]
    MissingProp(PropId),
    #[error("missing a prop in attribute update: {0} not found")]
    PropNotFound(String),
    #[error("missing a func in attribute update: {0} not found")]
    MissingFunc(String),
    #[error("invalid prop value; expected {0} but got {1}")]
    InvalidPropValue(String, serde_json::Value),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("validation resolver error: {0}")]
    ValidationResolver(#[from] ValidationResolverError),
    #[error("validation prototype error: {0}")]
    ValidationPrototype(#[from] ValidationPrototypeError),
    #[error("qualification view error: {0}")]
    QualificationView(#[from] QualificationError),
    #[error("resource error: {0}")]
    Resource(#[from] ResourceError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("workspace not found")]
    WorkspaceNotFound,
    #[error("organization not found")]
    OrganizationNotFound,
    #[error("billing account not found")]
    BillingAccountNotFound,
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("organization error: {0}")]
    Organization(#[from] OrganizationError),
    #[error("invalid json pointer: {0} for {1}")]
    BadJsonPointer(String, String),
    #[error("invalid AttributeReadContext: {0}")]
    BadAttributeReadContext(String),
}

pub type ComponentResult<T> = Result<T, ComponentError>;

const GET_RESOURCE: &str = include_str!("./queries/component_get_resource.sql");
const LIST_QUALIFICATIONS: &str = include_str!("./queries/component_list_qualifications.sql");
const LIST_CODE_GENERATED: &str = include_str!("./queries/component_list_code_generated.sql");
const LIST_FOR_RESOURCE_SYNC: &str = include_str!("./queries/component_list_for_resource_sync.sql");

pk!(ComponentPk);
pk!(ComponentId);

#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ComponentKind {
    Standard,
    Credential,
}

impl Default for ComponentKind {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pk: ComponentPk,
    id: ComponentId,
    kind: ComponentKind,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: Component,
    pk: ComponentPk,
    id: ComponentId,
    table_name: "components",
    history_event_label_base: "component",
    history_event_message_name: "Component"
}

impl Component {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new_for_schema_with_node(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        encryption_key: &EncryptionKey,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
        schema_id: &SchemaId,
    ) -> ComponentResult<(Self, Node)> {
        let mut schema_tenancy = tenancy.clone();
        schema_tenancy.universal = true;

        let schema = Schema::get_by_id(txn, &schema_tenancy, visibility, schema_id)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;

        let schema_variant_id = schema
            .default_schema_variant_id()
            .ok_or(ComponentError::SchemaVariantNotFound)?;

        Self::new_for_schema_variant_with_node(
            txn,
            nats,
            veritech,
            encryption_key,
            tenancy,
            visibility,
            history_actor,
            name,
            schema_variant_id,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new_for_schema_variant_with_node(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        encryption_key: &EncryptionKey,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
        schema_variant_id: &SchemaVariantId,
    ) -> ComponentResult<(Self, Node)> {
        let mut schema_tenancy = tenancy.clone();
        schema_tenancy.universal = true;

        let schema_variant =
            SchemaVariant::get_by_id(txn, &schema_tenancy, visibility, schema_variant_id)
                .await?
                .ok_or(ComponentError::SchemaVariantNotFound)?;
        let schema = schema_variant
            .schema(txn, visibility)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;

        let row = txn
            .query_one(
                "SELECT object FROM component_create_v1($1, $2, $3)",
                &[&tenancy, &visibility, &schema.component_kind().as_ref()],
            )
            .await?;

        let component: Component = standard_model::finish_create_from_row(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            row,
        )
        .await?;
        component
            .set_schema(txn, nats, visibility, history_actor, schema.id())
            .await?;
        component
            .set_schema_variant(txn, nats, visibility, history_actor, schema_variant.id())
            .await?;

        // Need to flesh out node so that the template data is also included in the node we
        // persist. But it isn't, - our node is anemic.
        let node = Node::new(
            txn,
            nats,
            &tenancy.into(),
            visibility,
            history_actor,
            &NodeKind::Component,
        )
        .await?;
        node.set_component(txn, nats, visibility, history_actor, component.id())
            .await?;

        // TODO: Eventually, we'll need the logic to be more complex than stuffing everything into the "production" system, but that's a problem for "a week or two from now" us.
        let mut systems =
            System::find_by_attr(txn, tenancy, visibility, "name", &"production").await?;
        let system = systems.pop().ok_or(ComponentError::SystemNotFound)?;
        let _edge = Edge::include_component_in_system(
            txn,
            nats,
            &tenancy.into(),
            visibility,
            history_actor,
            component.id(),
            system.id(),
        )
        .await?;

        // NOTE: We may want to be a bit smarter about when we create the Resource
        //       at some point in the future, by only creating it if there is also
        //       a ResourcePrototype for the Component's SchemaVariant.
        let _resource = Resource::new(
            txn,
            nats,
            &tenancy.into(),
            visibility,
            history_actor,
            component.id(),
            system.id(),
        )
        .await?;

        let name: &str = name.as_ref();
        component
            .set_prop_value_by_json_pointer(
                txn,
                nats,
                veritech,
                encryption_key,
                tenancy,
                history_actor,
                visibility,
                "/root/si/name",
                Some(name),
            )
            .await?;

        Ok((component, node))
    }

    #[instrument(skip_all)]
    #[allow(clippy::too_many_arguments)]
    pub async fn new_application_with_node(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        encryption_key: &EncryptionKey,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
    ) -> ComponentResult<(Self, Node)> {
        let read_tenancy = ReadTenancy::new_universal();

        let schema_variant_id = Schema::default_schema_variant_id_for_name(
            txn,
            &read_tenancy,
            visibility,
            "application",
        )
        .await?;

        let (component, node) = Self::new_for_schema_variant_with_node(
            txn,
            nats,
            veritech,
            encryption_key,
            tenancy,
            visibility,
            history_actor,
            name,
            &schema_variant_id,
        )
        .await?;
        Ok((component, node))
    }

    standard_model_accessor!(kind, Enum(ComponentKind), ComponentResult);

    standard_model_belongs_to!(
        lookup_fn: schema,
        set_fn: set_schema,
        unset_fn: unset_schema,
        table: "component_belongs_to_schema",
        model_table: "schemas",
        belongs_to_id: SchemaId,
        returns: Schema,
        result: ComponentResult,
    );

    standard_model_belongs_to!(
        lookup_fn: schema_variant,
        set_fn: set_schema_variant,
        unset_fn: unset_schema_variant,
        table: "component_belongs_to_schema_variant",
        model_table: "schema_variants",
        belongs_to_id: SchemaVariantId,
        returns: SchemaVariant,
        result: ComponentResult,
    );

    standard_model_has_many!(
        lookup_fn: node,
        table: "node_belongs_to_component",
        model_table: "nodes",
        returns: Node,
        result: ComponentResult,
    );

    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn update_prop_from_edit_field(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        encryption_key: &EncryptionKey,
        write_tenancy: &WriteTenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        component_id: ComponentId,
        prop_id: PropId,
        _edit_field_id: String,
        value: Option<serde_json::Value>,
        key: Option<String>,
    ) -> ComponentResult<()> {
        let read_tenancy = write_tenancy.clone_into_read_tenancy(txn).await?;
        let prop = Prop::get_by_id(txn, &(&read_tenancy).into(), visibility, &prop_id)
            .await?
            .ok_or(ComponentError::MissingProp(prop_id))?;
        let component = Self::get_by_id(txn, &(&read_tenancy).into(), visibility, &component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))?;

        // TODO: Eventually, we'll need the logic to be more complex than stuffing everything into
        // the "production" system, but that's a problem for "a week or two from now" us.
        let mut systems = System::find_by_attr(
            txn,
            &(&read_tenancy).into(),
            visibility,
            "name",
            &"production",
        )
        .await?;
        let system = systems.pop().ok_or(ComponentError::SystemNotFound)?;

        let (value, _attribute_resolver_id, created) = component
            .resolve_attribute(
                txn,
                nats,
                veritech.clone(),
                encryption_key,
                &write_tenancy.into(),
                visibility,
                history_actor,
                &prop,
                value,
                None,
                key,
                *system.id(),
            )
            .await?;

        component
            .check_validations(
                txn,
                nats,
                veritech.clone(),
                encryption_key,
                &write_tenancy.into(),
                visibility,
                history_actor,
                &prop,
                &value,
                created,
            )
            .await?;

        // Some qualifications depend on code generation, so we have to generate first
        component
            .generate_code(
                txn,
                nats,
                veritech.clone(),
                encryption_key,
                &write_tenancy.into(),
                visibility,
                history_actor,
                UNSET_ID_VALUE.into(), // TODO: properly obtain a system_id
            )
            .await?;

        component
            .check_qualifications(
                txn,
                nats,
                veritech,
                encryption_key,
                &write_tenancy.into(),
                visibility,
                history_actor,
                UNSET_ID_VALUE.into(), // TODO: properly obtain a system_id
            )
            .await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    #[async_recursion]
    /// Perform the actual value setting for a given prop. If the value passed in is empty, we
    /// greedily search for an attribute resolver to set the prop's default value, but "unsetting"
    /// is currently unsupported beyond the initial implementation for "PropKind::String".
    #[instrument(skip_all)]
    pub async fn resolve_attribute(
        &self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        encryption_key: &EncryptionKey,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        prop: &Prop,
        value: Option<serde_json::Value>,
        parent_attribute_resolver_id: Option<AttributeResolverId>,
        key: Option<String>,
        system_id: SystemId,
    ) -> ComponentResult<(Option<serde_json::Value>, AttributeResolverId, bool)> {
        let mut schema_tenancy = tenancy.clone();
        schema_tenancy.universal = true;

        // Allows for finding the default Attribute Resolver for a Prop
        let mut attribute_resolver_context = AttributeResolverContext::new();
        attribute_resolver_context.set_prop_id(*prop.id());

        fn as_type<T: serde::de::DeserializeOwned>(json: serde_json::Value) -> ComponentResult<T> {
            T::deserialize(&json).map_err(|_| {
                ComponentError::InvalidPropValue(std::any::type_name::<T>().to_owned(), json)
            })
        }

        async fn set_value<T: Serialize>(
            txn: &PgTxn<'_>,
            nats: &NatsTxn,
            veritech: veritech::Client,
            encryption_key: &EncryptionKey,
            tenancy: &Tenancy,
            visibility: &Visibility,
            history_actor: &HistoryActor,
            func_name: &str,
            args: T,
        ) -> ComponentResult<(Func, FuncBinding, bool)> {
            let mut schema_tenancy = tenancy.clone();
            schema_tenancy.universal = true;

            let func_name = func_name.to_owned();
            let mut funcs =
                Func::find_by_attr(txn, &schema_tenancy, visibility, "name", &func_name).await?;
            let func = funcs.pop().ok_or(ComponentError::MissingFunc(func_name))?;
            let (func_binding, created) = FuncBinding::find_or_create(
                txn,
                nats,
                &tenancy.into(),
                visibility,
                history_actor,
                serde_json::to_value(args)?,
                *func.id(),
                *func.backend_kind(),
            )
            .await?;

            // Note for future humans - if this isn't a built in, then we need to
            // think about execution time. Probably higher up than this? But just
            // an FYI.
            if created {
                func_binding
                    .execute(txn, nats, veritech, encryption_key)
                    .await?;
            }
            Ok((func, func_binding, created))
        }

        async fn unset_value(
            txn: &PgTxn<'_>,
            nats: &NatsTxn,
            veritech: veritech::Client,
            encryption_key: &EncryptionKey,
            tenancy: &Tenancy,
            visibility: &Visibility,
            history_actor: &HistoryActor,
            args_default: FuncBackendJsAttributeArgs,
            attribute_resolver_context: AttributeResolverContext,
        ) -> ComponentResult<(Func, FuncBinding, bool)> {
            let mut schema_tenancy = tenancy.clone();
            schema_tenancy.universal = true;

            if let Some(resolver) = AttributeResolver::find_for_context(
                txn,
                &schema_tenancy,
                visibility,
                attribute_resolver_context,
            )
            .await?
            {
                let func = Func::get_by_id(txn, &schema_tenancy, visibility, &resolver.func_id())
                    .await?
                    .ok_or_else(|| ComponentError::MissingFunc(resolver.func_id().to_string()))?;
                let (func_binding, created) = FuncBinding::find_or_create(
                    txn,
                    nats,
                    &tenancy.into(),
                    visibility,
                    history_actor,
                    serde_json::to_value(&args_default)?,
                    *func.id(),
                    *func.backend_kind(),
                )
                .await?;

                if created {
                    func_binding
                        .execute(txn, nats, veritech, encryption_key)
                        .await?;
                }

                Ok((func, func_binding, created))
            } else {
                set_value(
                    txn,
                    nats,
                    veritech,
                    encryption_key,
                    tenancy,
                    visibility,
                    history_actor,
                    "si:unset",
                    (),
                )
                .await
            }
        }

        // We shouldn't be leaking this value, because it may or may not be actually set. But
        // when you YOLO, YOLO hard. -- Adam
        let (func, func_binding, created) = match (prop.kind(), value.clone()) {
            (PropKind::Array, Some(value_json)) => {
                // FIXME(nick): handle nesting for Array.

                let value: Vec<serde_json::Value> = as_type(value_json)?;
                set_value(
                    txn,
                    nats,
                    veritech.clone(),
                    encryption_key,
                    tenancy,
                    visibility,
                    history_actor,
                    "si:setArray",
                    FuncBackendArrayArgs::new(value),
                )
                .await?
            }
            (PropKind::Boolean, Some(value_json)) => {
                let value: bool = as_type(value_json)?;
                set_value(
                    txn,
                    nats,
                    veritech.clone(),
                    encryption_key,
                    tenancy,
                    visibility,
                    history_actor,
                    "si:setBoolean",
                    FuncBackendBooleanArgs::new(value),
                )
                .await?
            }
            (PropKind::Integer, Some(value_json)) => {
                let value: i64 = as_type(value_json)?;
                set_value(
                    txn,
                    nats,
                    veritech.clone(),
                    encryption_key,
                    tenancy,
                    visibility,
                    history_actor,
                    "si:setInteger",
                    FuncBackendIntegerArgs::new(value),
                )
                .await?
            }
            (PropKind::Map, Some(value_json)) => {
                // FIXME(nick): handle nesting for Map.

                let value: serde_json::Map<String, serde_json::Value> = as_type(value_json)?;
                set_value(
                    txn,
                    nats,
                    veritech.clone(),
                    encryption_key,
                    tenancy,
                    visibility,
                    history_actor,
                    "si:setMap",
                    FuncBackendMapArgs::new(value),
                )
                .await?
            }
            (PropKind::Object, Some(value_json)) => {
                // FIXME(nick): deny objects that aren't empty here. The value must be empty for a PropObject.
                // FIXME(nick,jacob): add object nesting. This is incomplete!

                let value: serde_json::Map<String, serde_json::Value> = as_type(value_json)?;
                set_value(
                    txn,
                    nats,
                    veritech.clone(),
                    encryption_key,
                    tenancy,
                    visibility,
                    history_actor,
                    "si:setPropObject",
                    FuncBackendPropObjectArgs::new(value),
                )
                .await?
            }
            (PropKind::String, Some(value_json)) => {
                let value: String = as_type(value_json)?;

                set_value(
                    txn,
                    nats,
                    veritech.clone(),
                    encryption_key,
                    tenancy,
                    visibility,
                    history_actor,
                    "si:setString",
                    FuncBackendStringArgs::new(value),
                )
                .await?
            }
            (_, None) => {
                let args_default = FuncBackendJsAttributeArgs {
                    component: self
                        .veritech_attribute_resolver_component(
                            txn,
                            &schema_tenancy,
                            visibility,
                            system_id,
                        )
                        .await?,
                };
                unset_value(
                    txn,
                    nats,
                    veritech.clone(),
                    encryption_key,
                    tenancy,
                    visibility,
                    history_actor,
                    args_default,
                    attribute_resolver_context,
                )
                .await?
            }
        };

        let mut attribute_resolver_context = AttributeResolverContext::new();
        attribute_resolver_context.set_prop_id(*prop.id());
        attribute_resolver_context.set_component_id(*self.id());

        let parent_prop = prop.parent_prop(txn, visibility).await?;
        let attribute_resolver = if let (Some(parent_prop), Some(parent_attribute_resolver_id)) =
            (parent_prop, parent_attribute_resolver_id)
        {
            let attribute_resolver = AttributeResolver::new(
                txn,
                nats,
                tenancy,
                visibility,
                history_actor,
                *func.id(),
                *func_binding.id(),
                attribute_resolver_context,
                key.clone(),
            )
            .await?;

            attribute_resolver
                .set_parent_attribute_resolver(
                    txn,
                    nats,
                    visibility,
                    history_actor,
                    parent_attribute_resolver_id,
                )
                .await?;

            // Next we make sure that our parent AttributeResolver resolves to an appropriate
            // "empty" value for the PropKind, as long as we have a value.
            // In the case where our value is `unset`, and there are no other children of our
            // parent with a value (we were the last child that had a non-`unset` value), then
            // we want to set our parent's value to be `unset` as well.

            // TODO: Eventually, we'll need the logic to be more complex than stuffing everything into the "production" system, but that's a problem for "a week or two from now" us.
            let mut systems =
                System::find_by_attr(txn, tenancy, visibility, "name", &"production").await?;
            let system = systems.pop().ok_or(ComponentError::SystemNotFound)?;
            let current_parent_value = AttributeResolver::find_value_for_prop_and_component(
                txn,
                tenancy,
                visibility,
                *parent_prop.id(),
                *self.id(),
                *system.id(),
            )
            .await?;
            let (should_change_parent, new_parent_prop_value) = if value.is_some() {
                match parent_prop.kind() {
                    PropKind::Array => (true, Some(serde_json::json![[]])),
                    PropKind::Map | PropKind::Object => (true, Some(serde_json::json![{}])),
                    _ => (false, None),
                }
            } else {
                // If we're setting an AttributeResolver to Unset, we should only ever set the parent AttributeResolver
                // to unset if this is the only remaining AttributeResolver that wasn't already Unset.
                if AttributeResolver::any_siblings_are_set(
                    txn,
                    tenancy,
                    visibility,
                    *attribute_resolver.id(),
                )
                .await?
                {
                    (false, None)
                } else {
                    (true, None)
                }
            };
            if should_change_parent
                && current_parent_value.value() != new_parent_prop_value.as_ref()
            {
                let parent_attribute_resolver = AttributeResolver::get_by_id(
                    txn,
                    tenancy,
                    visibility,
                    &parent_attribute_resolver_id,
                )
                .await?
                .ok_or(ComponentError::MissingAttributeResolver(
                    parent_attribute_resolver_id,
                ))?;
                let parent_parent_attribute_resolver_id = parent_attribute_resolver
                    .parent_attribute_resolver(txn, visibility)
                    .await?
                    .map(|ar| *ar.id());

                self.resolve_attribute(
                    txn,
                    nats,
                    veritech,
                    encryption_key,
                    tenancy,
                    visibility,
                    history_actor,
                    &parent_prop,
                    new_parent_prop_value,
                    parent_parent_attribute_resolver_id,
                    key,
                    *system.id(),
                )
                .await?;
            }

            attribute_resolver
        } else {
            AttributeResolver::upsert(
                txn,
                nats,
                tenancy,
                visibility,
                history_actor,
                *func.id(),
                *func_binding.id(),
                attribute_resolver_context,
                key,
            )
            .await?
        };

        Ok((value, *attribute_resolver.id(), created))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn check_validations(
        &self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        encryption_key: &EncryptionKey,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        prop: &Prop,
        value: &Option<serde_json::Value>,
        created: bool,
    ) -> ComponentResult<()> {
        let validators = ValidationPrototype::find_for_prop(
            txn,
            &tenancy.clone_into_read_tenancy(txn).await?,
            visibility,
            *prop.id(),
            UNSET_ID_VALUE.into(),
        )
        .await?;

        for validator in validators {
            let func = Func::get_by_id(txn, tenancy, visibility, &validator.func_id())
                .await?
                .ok_or_else(|| ComponentError::MissingFunc(validator.func_id().to_string()))?;
            let func_binding = match func.backend_kind() {
                FuncBackendKind::ValidateStringValue => {
                    let mut args =
                        FuncBackendValidateStringValueArgs::deserialize(validator.args())?;
                    if let Some(json_value) = value {
                        if json_value.is_string() {
                            args.value = Some(json_value.to_string());
                        } else {
                            return Err(ComponentError::InvalidPropValue(
                                "String".to_string(),
                                json_value.clone(),
                            ));
                        }
                    } else {
                        // TODO: This might not be quite the right error to return here if we got a None.
                        return Err(ComponentError::MissingProp(*prop.id()));
                    };
                    let args_json = serde_json::to_value(args)?;
                    let (func_binding, binding_created) = FuncBinding::find_or_create(
                        txn,
                        nats,
                        &tenancy.into(),
                        visibility,
                        history_actor,
                        args_json,
                        *func.id(),
                        *func.backend_kind(),
                    )
                    .await?;
                    // Note for future humans - if this isn't a built in, then we need to
                    // think about execution time. Probably higher up than this? But just
                    // an FYI.
                    if binding_created {
                        func_binding
                            .execute(txn, nats, veritech.clone(), encryption_key)
                            .await?;
                    }
                    func_binding
                }
                kind => unimplemented!("Validator Backend not supported yet: {}", kind),
            };

            if created {
                let mut existing_validation_resolvers = ValidationResolver::find_for_prototype(
                    txn,
                    &tenancy.clone_into_read_tenancy(txn).await?,
                    visibility,
                    validator.id(),
                )
                .await?;

                // If we don't have one, create the validation resolver. If we do, update the
                // func binding id to point to the new value. Interesting to think about
                // garbage collecting the left over funcbinding + func result value?
                if let Some(mut validation_resolver) = existing_validation_resolvers.pop() {
                    validation_resolver
                        .set_func_binding_id(
                            txn,
                            nats,
                            visibility,
                            history_actor,
                            *func_binding.id(),
                        )
                        .await?;
                } else {
                    let mut validation_resolver_context = ValidationResolverContext::new();
                    validation_resolver_context.set_prop_id(*prop.id());
                    validation_resolver_context.set_component_id(*self.id());
                    ValidationResolver::new(
                        txn,
                        nats,
                        &tenancy.into(),
                        visibility,
                        history_actor,
                        *validator.id(),
                        *func.id(),
                        *func_binding.id(),
                        validation_resolver_context,
                    )
                    .await?;
                }
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn check_qualifications(
        &self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        encryption_key: &EncryptionKey,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        system_id: SystemId,
    ) -> ComponentResult<()> {
        let mut schema_tenancy = tenancy.clone();
        schema_tenancy.universal = true;

        let schema = self
            .schema_with_tenancy(txn, &schema_tenancy, visibility)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = self
            .schema_variant_with_tenancy(txn, &schema_tenancy, visibility)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;

        let qualification_prototypes = QualificationPrototype::find_for_component(
            txn,
            &tenancy.clone_into_read_tenancy(txn).await?,
            visibility,
            *self.id(),
            *schema.id(),
            *schema_variant.id(),
            system_id,
        )
        .await?;

        for prototype in qualification_prototypes {
            let func = Func::get_by_id(txn, &schema_tenancy, visibility, &prototype.func_id())
                .await?
                .ok_or_else(|| ComponentError::MissingFunc(prototype.func_id().to_string()))?;

            let args = FuncBackendJsQualificationArgs {
                component: self
                    .veritech_qualification_check_component(
                        txn,
                        &schema_tenancy,
                        visibility,
                        system_id,
                    )
                    .await?,
            };

            let json_args = serde_json::to_value(args)?;
            let (func_binding, created) = FuncBinding::find_or_create(
                txn,
                nats,
                &tenancy.into(),
                visibility,
                history_actor,
                json_args,
                prototype.func_id(),
                *func.backend_kind(),
            )
            .await?;

            if created {
                // Note for future humans - if this isn't a built in, then we need to
                // think about execution time. Probably higher up than this? But just
                // an FYI.
                func_binding
                    .execute(txn, nats, veritech.clone(), encryption_key)
                    .await?;

                let mut existing_resolvers =
                    QualificationResolver::find_for_prototype_and_component(
                        txn,
                        &tenancy.clone_into_read_tenancy(txn).await?,
                        visibility,
                        prototype.id(),
                        self.id(),
                    )
                    .await?;

                // If we do not have one, create the qualification resolver. If we do, update the
                // func binding id to point to the new value.
                if let Some(mut resolver) = existing_resolvers.pop() {
                    resolver
                        .set_func_binding_id(
                            txn,
                            nats,
                            visibility,
                            history_actor,
                            *func_binding.id(),
                        )
                        .await?;
                } else {
                    let mut resolver_context = QualificationResolverContext::new();
                    resolver_context.set_component_id(*self.id());
                    QualificationResolver::new(
                        txn,
                        nats,
                        &tenancy.into(),
                        visibility,
                        history_actor,
                        *prototype.id(),
                        *func.id(),
                        *func_binding.id(),
                        resolver_context,
                    )
                    .await?;
                }
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn generate_code(
        &self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        encryption_key: &EncryptionKey,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        system_id: SystemId,
    ) -> ComponentResult<()> {
        let mut schema_tenancy = tenancy.clone();
        schema_tenancy.universal = true;

        let schema = self
            .schema_with_tenancy(txn, &schema_tenancy, visibility)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = self
            .schema_variant_with_tenancy(txn, &schema_tenancy, visibility)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;

        let code_generation_prototypes = CodeGenerationPrototype::find_for_component(
            txn,
            &tenancy.clone_into_read_tenancy(txn).await?,
            visibility,
            *self.id(),
            *schema.id(),
            *schema_variant.id(),
            system_id,
        )
        .await?;

        for prototype in code_generation_prototypes {
            let func = Func::get_by_id(txn, &schema_tenancy, visibility, &prototype.func_id())
                .await?
                .ok_or_else(|| ComponentError::MissingFunc(prototype.func_id().to_string()))?;

            let args = FuncBackendJsCodeGenerationArgs {
                component: self
                    .veritech_code_generation_component(txn, &schema_tenancy, visibility, system_id)
                    .await?,
            };
            let json_args = serde_json::to_value(args)?;

            let (func_binding, created) = FuncBinding::find_or_create(
                txn,
                nats,
                &tenancy.into(),
                visibility,
                history_actor,
                json_args,
                prototype.func_id(),
                *func.backend_kind(),
            )
            .await?;

            if created {
                // Note for future humans - if this isn't a built in, then we need to
                // think about execution time. Probably higher up than this? But just
                // an FYI.
                func_binding
                    .execute(txn, nats, veritech.clone(), encryption_key)
                    .await?;

                let mut existing_resolvers =
                    CodeGenerationResolver::find_for_prototype_and_component(
                        txn,
                        &tenancy.clone_into_read_tenancy(txn).await?,
                        visibility,
                        prototype.id(),
                        self.id(),
                    )
                    .await?;

                // If we do not have one, create the code generation resolver. If we do, update the
                // func binding id to point to the new value.
                if let Some(mut resolver) = existing_resolvers.pop() {
                    resolver
                        .set_func_binding_id(
                            txn,
                            nats,
                            visibility,
                            history_actor,
                            *func_binding.id(),
                        )
                        .await?;
                } else {
                    let mut resolver_context = CodeGenerationResolverContext::new();
                    resolver_context.set_component_id(*self.id());
                    let _resolver = CodeGenerationResolver::new(
                        txn,
                        nats,
                        &tenancy.into(),
                        visibility,
                        history_actor,
                        *prototype.id(),
                        *func.id(),
                        *func_binding.id(),
                        resolver_context,
                    )
                    .await?;
                }
            }
        }

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn list_validations_as_qualification_for_component_id(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ComponentResult<QualificationView> {
        let validation_field_values = ValidationResolver::list_values_for_component(
            txn,
            &tenancy.clone_into_read_tenancy(txn).await?,
            visibility,
            component_id,
            system_id,
        )
        .await?;

        let mut validation_errors: Vec<(Prop, Vec<ValidationError>)> = Vec::new();
        for (prop, field_value) in validation_field_values.into_iter() {
            if let Some(value_json) = field_value.value() {
                // This clone shouldn't be necessary, but we have no way to get to the owned value -- Adam
                let internal_validation_errors: Vec<ValidationError> =
                    serde_json::from_value(value_json.clone())?;
                validation_errors.push((prop, internal_validation_errors));
            }
        }
        let qualification_view = QualificationView::new_for_validation_errors(validation_errors);
        Ok(qualification_view)
    }

    #[instrument(skip_all)]
    pub async fn list_code_generated_by_component_id(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ComponentResult<Vec<veritech::CodeGenerated>> {
        let mut results: Vec<veritech::CodeGenerated> = Vec::new();

        let rows = txn
            .query(
                LIST_CODE_GENERATED,
                &[&tenancy, &visibility, &component_id, &system_id],
            )
            .await?;
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("object")?;
            let func_binding_return_value: FuncBindingReturnValue = serde_json::from_value(json)?;
            let value = func_binding_return_value
                .value()
                .ok_or(ComponentError::CodeGeneratedNotFound)?;
            let code_generated = veritech::CodeGenerated::deserialize(value)?;
            results.push(code_generated);
        }
        Ok(results)
    }

    #[instrument(skip_all)]
    pub async fn list_qualifications(
        &self,
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        system_id: SystemId,
    ) -> ComponentResult<Vec<QualificationView>> {
        Self::list_qualifications_by_component_id(txn, tenancy, visibility, *self.id(), system_id)
            .await
    }

    #[instrument(skip_all)]
    pub async fn list_qualifications_by_component_id(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ComponentResult<Vec<QualificationView>> {
        let mut results: Vec<QualificationView> = Vec::new();

        // This is the "All Fields Valid" universal qualification
        let validation_qualification = Self::list_validations_as_qualification_for_component_id(
            txn,
            tenancy,
            visibility,
            component_id,
            system_id,
        )
        .await?;
        results.push(validation_qualification);

        let rows = txn
            .query(
                LIST_QUALIFICATIONS,
                &[&tenancy, &visibility, &component_id, &system_id],
            )
            .await?;
        let no_qualification_results = rows.is_empty();
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("object")?;
            let func_binding_return_value: FuncBindingReturnValue = serde_json::from_value(json)?;
            let mut qual_view = QualificationView::new_for_func_binding_return_value(
                txn,
                func_binding_return_value,
            )
            .await?;
            let title: String = row.try_get("title")?;
            let link: Option<String> = row.try_get("link")?;
            qual_view.title = title;
            qual_view.link = link;
            results.push(qual_view);
        }
        // This is inefficient, but effective
        if no_qualification_results {
            let component = Self::get_by_id(txn, tenancy, visibility, &component_id)
                .await?
                .ok_or(ComponentError::NotFound(component_id))?;
            let mut schema_tenancy = tenancy.clone();
            schema_tenancy.universal = true;
            let schema = component
                .schema_with_tenancy(txn, tenancy, visibility)
                .await?
                .ok_or(ComponentError::SchemaNotFound)?;
            let schema_variant = component
                .schema_variant_with_tenancy(txn, tenancy, visibility)
                .await?
                .ok_or(ComponentError::SchemaVariantNotFound)?;
            let prototypes = QualificationPrototype::find_for_component(
                txn,
                &tenancy.clone_into_read_tenancy(txn).await?,
                visibility,
                component_id,
                *schema.id(),
                *schema_variant.id(),
                system_id,
            )
            .await?;
            for prototype in prototypes.into_iter() {
                let qual_view = QualificationView::new_for_qualification_prototype(prototype);
                results.push(qual_view);
            }
        }
        Ok(results)
    }

    #[instrument(skip_all)]
    pub async fn get_resource_by_component_and_system(
        txn: &PgTxn<'_>,
        read_tenancy: &ReadTenancy,
        visibility: &Visibility,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ComponentResult<Option<ResourceView>> {
        let resource = Resource::get_by_component_id_and_system_id(
            txn,
            read_tenancy,
            visibility,
            &component_id,
            &system_id,
        )
        .await?;
        let resource = match resource {
            Some(r) => r,
            None => return Ok(None),
        };

        let row = txn
            .query_opt(
                GET_RESOURCE,
                &[read_tenancy, &visibility, &component_id, &system_id],
            )
            .await?;

        let json: Option<serde_json::Value> = row.map(|row| row.try_get("object")).transpose()?;

        let func_binding_return_value: Option<FuncBindingReturnValue> =
            json.map(serde_json::from_value).transpose()?;
        let res_view = ResourceView::from((resource, func_binding_return_value));

        Ok(Some(res_view))
    }

    pub async fn veritech_attribute_resolver_component(
        &self,
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        system_id: SystemId,
    ) -> ComponentResult<veritech::ResolverFunctionComponent> {
        let read_tenancy = tenancy.clone_into_read_tenancy(txn).await?;
        let parent_ids =
            Edge::find_component_configuration_parents(txn, &read_tenancy, visibility, self.id())
                .await?;
        let mut parents = Vec::with_capacity(parent_ids.len());
        for id in parent_ids {
            let view = ComponentView::for_component_and_system(
                txn,
                &read_tenancy,
                visibility,
                id,
                system_id,
            )
            .await?;
            parents.push(veritech::ComponentView::from(view));
        }

        let component = veritech::ResolverFunctionComponent {
            data: ComponentView::for_component_and_system(
                txn,
                &read_tenancy,
                visibility,
                *self.id(),
                system_id,
            )
            .await?
            .into(),
            parents,
        };
        Ok(component)
    }

    pub async fn veritech_code_generation_component(
        &self,
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        system_id: SystemId,
    ) -> ComponentResult<ComponentView> {
        let read_tenancy = tenancy.clone_into_read_tenancy(txn).await?;
        let component = ComponentView::for_component_and_system(
            txn,
            &read_tenancy,
            visibility,
            *self.id(),
            system_id,
        )
        .await?;
        Ok(component)
    }

    pub async fn veritech_resource_sync_component(
        &self,
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        system_id: SystemId,
    ) -> ComponentResult<ComponentView> {
        let read_tenancy = tenancy.clone_into_read_tenancy(txn).await?;
        let component = ComponentView::for_component_and_system(
            txn,
            &read_tenancy,
            visibility,
            *self.id(),
            system_id,
        )
        .await?;
        Ok(component)
    }

    pub async fn veritech_qualification_check_component(
        &self,
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        system_id: SystemId,
    ) -> ComponentResult<veritech::QualificationCheckComponent> {
        let read_tenancy = tenancy.clone_into_read_tenancy(txn).await?;

        let parent_ids =
            Edge::find_component_configuration_parents(txn, &read_tenancy, visibility, self.id())
                .await?;

        let mut parents = Vec::new();
        for id in parent_ids {
            let view = ComponentView::for_component_and_system(
                txn,
                &read_tenancy,
                visibility,
                id,
                system_id,
            )
            .await?;
            parents.push(veritech::ComponentView::from(view));
        }

        let qualification_view = veritech::QualificationCheckComponent {
            data: ComponentView::for_component_and_system(
                txn,
                &read_tenancy,
                visibility,
                *self.id(),
                system_id,
            )
            .await?
            .into(),
            codes: Self::list_code_generated_by_component_id(
                txn,
                &(&read_tenancy).into(),
                visibility,
                *self.id(),
                system_id,
            )
            .await?,
            parents,
        };
        Ok(qualification_view)
    }

    #[instrument(skip_all)]
    pub async fn list_for_resource_sync(txn: &PgTxn<'_>) -> ComponentResult<Vec<Component>> {
        let visibility = Visibility::new_head(false);
        let rows = txn.query(LIST_FOR_RESOURCE_SYNC, &[&visibility]).await?;
        let results = standard_model::objects_from_rows(rows)?;
        Ok(results)
    }

    #[instrument(skip_all)]
    pub async fn sync_resource(
        &self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        encryption_key: &EncryptionKey,
        history_actor: &HistoryActor,
        system_id: SystemId,
    ) -> ComponentResult<()> {
        // Note(paulo): we don't actually care about the Resource here, we only care about the ResourcePrototype, is this wrong?

        let mut schema_tenancy = self.tenancy.clone();
        schema_tenancy.universal = true;

        let write_tenancy = (&self.tenancy).into();
        let read_tenancy = self.tenancy.clone_into_read_tenancy(txn).await?;

        let schema = self
            .schema_with_tenancy(txn, &schema_tenancy, &self.visibility)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = self
            .schema_variant_with_tenancy(txn, &schema_tenancy, &self.visibility)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;

        let resource_prototype = ResourcePrototype::get_for_component(
            txn,
            &read_tenancy,
            &self.visibility,
            *self.id(),
            *schema.id(),
            *schema_variant.id(),
            system_id,
        )
        .await?;

        if let Some(prototype) = resource_prototype {
            let func =
                Func::get_by_id(txn, &schema_tenancy, &self.visibility, &prototype.func_id())
                    .await?
                    .ok_or_else(|| ComponentError::MissingFunc(prototype.func_id().to_string()))?;

            let args = FuncBackendJsResourceSyncArgs {
                component: self
                    .veritech_resource_sync_component(
                        txn,
                        &schema_tenancy,
                        &self.visibility,
                        system_id,
                    )
                    .await?,
            };

            let (func_binding, _created) = FuncBinding::find_or_create(
                txn,
                nats,
                &write_tenancy,
                &self.visibility,
                history_actor,
                serde_json::to_value(args)?,
                prototype.func_id(),
                *func.backend_kind(),
            )
            .await?;

            // Note: We need to execute the same func binding a bunch of times
            func_binding
                .execute(txn, nats, veritech.clone(), encryption_key)
                .await?;

            // Note for future humans - if this isn't a built in, then we need to
            // think about execution time. Probably higher up than this? But just
            // an FYI.
            let existing_resolver = ResourceResolver::get_for_prototype_and_component(
                txn,
                &read_tenancy,
                &self.visibility,
                prototype.id(),
                self.id(),
            )
            .await?;

            // If we do not have one, create the resource resolver. If we do, update the
            // func binding id to point to the new value.
            let mut resolver = if let Some(resolver) = existing_resolver {
                resolver
            } else {
                let mut resolver_context = ResourceResolverContext::new();
                resolver_context.set_component_id(*self.id());
                ResourceResolver::new(
                    txn,
                    nats,
                    &write_tenancy,
                    &self.visibility,
                    history_actor,
                    *prototype.id(),
                    *func.id(),
                    *func_binding.id(),
                    resolver_context,
                )
                .await?
            };
            resolver
                .set_func_binding_id(
                    txn,
                    nats,
                    &self.visibility,
                    history_actor,
                    *func_binding.id(),
                )
                .await?;
        }

        let workspace_ids = self.tenancy.workspace_ids.clone();
        if workspace_ids.is_empty() {
            return Err(ComponentError::WorkspaceNotFound);
        }
        let billing_account_ids = ReadTenancy::new_workspace(txn, workspace_ids)
            .await?
            .billing_accounts()
            .to_owned();
        if billing_account_ids.is_empty() {
            warn!("No billing accounts found for organization");
            return Err(ComponentError::BillingAccountNotFound);
        } else {
            WsEvent::resource_synced(*self.id(), system_id, billing_account_ids, history_actor)
                .publish(nats)
                .await?;
        }

        Ok(())
    }

    // Note: Won't work for arrays and maps
    #[instrument(skip_all)]
    #[allow(clippy::too_many_arguments)]
    pub async fn set_prop_value_by_json_pointer<T: Serialize + std::fmt::Debug>(
        &self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        encryption_key: &EncryptionKey,
        tenancy: &Tenancy,
        history_actor: &HistoryActor,
        visibility: &Visibility,
        json_pointer: &str,
        value: Option<T>,
    ) -> ComponentResult<Option<T>> {
        let prop = self
            .find_prop_by_json_pointer(txn, tenancy, visibility, json_pointer)
            .await?;

        if let Some(prop) = prop {
            // This was copied from sdf's update_from_edit_field service
            Self::update_prop_from_edit_field(
                txn,
                nats,
                veritech,
                encryption_key,
                &tenancy.into(),
                visibility,
                history_actor,
                *self.id(),
                *prop.id(),
                json_pointer[1..].replace('/', "."),
                value.as_ref().map(serde_json::to_value).transpose()?,
                None, // TODO: Eventually, pass the key! -- Adam
            )
            .await?;
            Ok(value)
        } else {
            Err(ComponentError::PropNotFound(json_pointer.to_owned()))
        }
    }

    #[instrument(skip_all)]
    pub async fn find_prop_by_json_pointer(
        &self,
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        json_pointer: &str,
    ) -> ComponentResult<Option<Prop>> {
        let mut schema_tenancy = tenancy.clone();
        schema_tenancy.universal = true;
        let schema_variant = self
            .schema_variant_with_tenancy(txn, &schema_tenancy, visibility)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;

        let mut hierarchy = json_pointer.split('/');
        hierarchy.next(); // Ignores empty part

        let mut next = match hierarchy.next() {
            Some(n) => n,
            None => return Ok(None),
        };

        let mut work_queue = schema_variant.props(txn, visibility).await?;
        while let Some(prop) = work_queue.pop() {
            if prop.name() == next {
                next = match hierarchy.next() {
                    Some(n) => n,
                    None => return Ok(Some(prop)),
                };
                work_queue.clear();
                work_queue.extend(prop.child_props(txn, &schema_tenancy, visibility).await?);
            }
        }
        Ok(None)
    }

    #[instrument(skip_all)]
    pub async fn find_prop_value_by_json_pointer<
        T: serde::de::DeserializeOwned + std::fmt::Debug,
    >(
        &self,
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        json_pointer: &str,
    ) -> ComponentResult<Option<T>> {
        let prop = self
            .find_prop_by_json_pointer(txn, tenancy, visibility, json_pointer)
            .await?;
        if let Some(prop) = prop {
            // Copied from edit_field_for_prop, this is bad tho
            let system_id = UNSET_ID_VALUE.into();
            match AttributeResolver::find_value_for_prop_and_component(
                txn,
                tenancy,
                visibility,
                *prop.id(),
                *self.id(),
                system_id,
            )
            .await
            {
                Ok(v) => Ok(v.value().cloned().map(serde_json::from_value).transpose()?),
                Err(e) => {
                    dbg!("missing attribute resolver; might be fine, might be a bug! who knows? only god.");
                    dbg!(&e);
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl EditFieldAble for Component {
    type Id = ComponentId;
    type Error = ComponentError;

    async fn get_edit_fields(
        txn: &PgTxn<'_>,
        read_tenancy: &ReadTenancy,
        visibility: &Visibility,
        id: &ComponentId,
    ) -> ComponentResult<EditFields> {
        let head_visibility = Visibility::new_head(visibility.deleted);
        let change_set_visibility =
            Visibility::new_change_set(visibility.change_set_pk, visibility.deleted);

        let component = Self::get_by_id(txn, &read_tenancy.into(), visibility, id)
            .await?
            .ok_or(ComponentError::NotFound(*id))?;

        let mut edit_fields = vec![];
        let schema_variant = component
            .schema_variant_with_tenancy(txn, &read_tenancy.into(), visibility)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;

        let props = schema_variant.props(txn, visibility).await?;
        for prop in &props {
            // TODO: remove this as soon as edit fields are implemented
            // But for now we need the boilerplate to setup the props even if not editable
            // So it was breaking some tests
            if *prop.kind() == PropKind::Array || *prop.kind() == PropKind::Map {
                continue;
            }

            // schema_variant.props returns all props, even if not root, this is a hotfix
            if prop.parent_prop(txn, visibility).await?.is_some() {
                continue;
            }

            edit_fields.push(
                edit_field_for_prop(
                    txn,
                    &read_tenancy.into(),
                    visibility,
                    &head_visibility,
                    &change_set_visibility,
                    prop,
                    &component,
                    None,
                )
                .await?,
            );
        }

        Ok(edit_fields)
    }

    async fn update_from_edit_field(
        _txn: &PgTxn<'_>,
        _nats: &NatsTxn,
        _veritech: veritech::Client,
        _encryption_key: &EncryptionKey,
        _write_tenancy: &WriteTenancy,
        _visibility: &Visibility,
        _history_actor: &HistoryActor,
        _id: Self::Id,
        edit_field_id: String,
        _value: Option<serde_json::Value>,
    ) -> ComponentResult<()> {
        Err(EditFieldError::invalid_field(edit_field_id).into())
    }
}

#[allow(clippy::too_many_arguments)]
#[async_recursion]
async fn edit_field_for_prop(
    txn: &PgTxn<'_>,
    tenancy: &Tenancy,
    visibility: &Visibility,
    head_visibility: &Visibility,
    change_set_visibility: &Visibility,
    prop: &Prop,
    component: &Component,
    edit_field_path: Option<Vec<String>>,
) -> ComponentResult<EditField> {
    let system_id = UNSET_ID_VALUE.into();
    let current_value: Option<FuncBindingReturnValue> =
        match AttributeResolver::find_value_for_prop_and_component(
            txn,
            tenancy,
            visibility,
            *prop.id(),
            *component.id(),
            system_id,
        )
        .await
        {
            Ok(v) => Some(v),
            Err(e) => {
                dbg!("missing attribute resolver; might be fine, might be a bug! who knows? only god.");
                dbg!(&e);
                None
            }
        };
    let head_value: Option<FuncBindingReturnValue> = if visibility.in_change_set() {
        match AttributeResolver::find_value_for_prop_and_component(
            txn,
            tenancy,
            head_visibility,
            *prop.id(),
            *component.id(),
            system_id,
        )
        .await
        {
            Ok(v) => Some(v),
            Err(e) => {
                dbg!("missing attribute resolver; might be fine, might be a bug! who knows? only god.");
                dbg!(&e);
                None
            }
        }
    } else {
        None
    };
    let change_set_value: Option<FuncBindingReturnValue> = if visibility.in_change_set() {
        match AttributeResolver::find_value_for_prop_and_component(
            txn,
            tenancy,
            change_set_visibility,
            *prop.id(),
            *component.id(),
            system_id,
        )
        .await
        {
            Ok(v) => Some(v),
            Err(e) => {
                dbg!("missing attribute resolver; might be fine, might be a bug! who knows? only god.");
                dbg!(&e);
                None
            }
        }
    } else {
        None
    };

    let field_name = prop.name();
    let object_kind = EditFieldObjectKind::ComponentProp;

    fn extract_value(fbrv: &FuncBindingReturnValue) -> Option<&serde_json::Value> {
        fbrv.value()
    }

    let (value, visibility_diff) = value_and_visibility_diff_json_option(
        visibility,
        current_value.as_ref(),
        extract_value,
        head_value.as_ref(),
        change_set_value.as_ref(),
    )?;

    let mut validation_errors = Vec::new();
    let validation_field_values = ValidationResolver::find_values_for_prop_and_component(
        txn,
        &tenancy.clone_into_read_tenancy(txn).await?,
        visibility,
        *prop.id(),
        *component.id(),
        system_id,
    )
    .await?;
    for field_value in validation_field_values.into_iter() {
        if let Some(value_json) = field_value.value() {
            // This clone shouldn't be necessary, but we have no way to get to the owned value -- Adam
            let mut validation_error: Vec<ValidationError> =
                serde_json::from_value(value_json.clone())?;
            validation_errors.append(&mut validation_error);
        }
    }

    let current_edit_field_path = match edit_field_path {
        None => vec!["properties".to_owned()],
        Some(path) => path,
    };
    let mut edit_field_path_for_children = current_edit_field_path.clone();
    edit_field_path_for_children.push(field_name.to_string());

    let widget = match prop.widget_kind() {
        WidgetKind::SecretSelect => {
            let mut entries = Vec::new();
            let secrets = Secret::list(txn, tenancy, visibility).await?;

            for secret in secrets.into_iter() {
                entries.push(LabelEntry::new(
                    secret.name(),
                    serde_json::json!(i64::from(*secret.id())),
                ));
            }
            Widget::Select(SelectWidget::new(LabelList::new(entries), None))
        }
        WidgetKind::Text => Widget::Text(TextWidget::new()),
        WidgetKind::Array | WidgetKind::Header => {
            // NOTE: This ends up being ugly, and double checking what prop.kind() is
            //       to avoid doing the child prop lookup if we're building the Widget
            //       for a PropKind that "can't" have children. It may be worth taking
            //       the hit and always looking up what the children are, even if
            //       we're never going to use them, just to make this arm of the match
            //       less gross.
            let mut child_edit_fields = vec![];
            for child_prop in prop.child_props(txn, tenancy, visibility).await? {
                // TODO: remove this as soon as edit fields are implemented
                // But for now we need the boilerplate to setup the props even if not editable
                // So it was breaking some tests
                if *child_prop.kind() == PropKind::Array || *child_prop.kind() == PropKind::Map {
                    continue;
                }

                child_edit_fields.push(
                    edit_field_for_prop(
                        txn,
                        tenancy,
                        visibility,
                        head_visibility,
                        change_set_visibility,
                        &child_prop,
                        component,
                        Some(edit_field_path_for_children.clone()),
                    )
                    .await?,
                );
            }

            if *prop.kind() == PropKind::Array {
                todo!("Need to handle Array props");
            } else if *prop.kind() == PropKind::Map {
                todo!("Need to handle Map props");
            } else {
                // Only option left is PropKind::Object
                Widget::Header(HeaderWidget::new(child_edit_fields))
            }
        }
        WidgetKind::Checkbox => Widget::Checkbox(CheckboxWidget::new()),
    };

    let mut edit_field = EditField::new(
        field_name,
        current_edit_field_path,
        object_kind,
        *component.id(),
        (*prop.kind()).into(),
        widget,
        value,
        visibility_diff,
        validation_errors,
    );
    edit_field.set_baggage(EditFieldBaggage::ComponentProp(
        EditFieldBaggageComponentProp {
            prop_id: *prop.id(),
            system_id: None,
        },
    ));

    Ok(edit_field)
}
