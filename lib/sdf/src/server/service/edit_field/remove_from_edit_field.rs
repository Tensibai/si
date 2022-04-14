use axum::Json;
use dal::{
    edit_field::{EditFieldAble, EditFieldBaggage, EditFieldObjectKind},
    schema::{self, SchemaVariant},
    socket::Socket,
    AttributeContext, Component, Prop, QualificationCheck, Schema, Visibility, WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::{EditFieldError, EditFieldResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoveFromEditFieldRequest {
    pub object_kind: EditFieldObjectKind,
    pub object_id: i64,
    pub edit_field_id: String,
    pub baggage: Option<EditFieldBaggage>,
    pub workspace_id: Option<WorkspaceId>,
    pub attribute_context: Option<AttributeContext>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoveFromEditFieldResponse {
    pub success: bool,
}

/// This function is very similar to [`crate::server::service::edit_field::update_from_edit_field::update_from_edit_field()`],
/// but instead of using a value in the request payload, [`None`] is used for the value in the
/// underlying update functions.
pub async fn remove_from_edit_field(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<RemoveFromEditFieldRequest>,
) -> EditFieldResult<Json<RemoveFromEditFieldResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    match request.object_kind {
        EditFieldObjectKind::Component => {
            Component::update_from_edit_field(
                &ctx,
                request.object_id.into(),
                request.edit_field_id,
                None,
            )
            .await?
        }
        EditFieldObjectKind::ComponentProp => {
            let baggage = request.baggage.ok_or(EditFieldError::MissingBaggage)?;
            let attribute_context = request
                .attribute_context
                .ok_or(EditFieldError::MissingAttributeContext)?;
            Component::update_from_edit_field_with_baggage(&ctx, None, attribute_context, baggage)
                .await?
        }
        EditFieldObjectKind::Prop => {
            Prop::update_from_edit_field(
                &ctx,
                request.object_id.into(),
                request.edit_field_id,
                None,
            )
            .await?
        }
        EditFieldObjectKind::QualificationCheck => {
            QualificationCheck::update_from_edit_field(
                &ctx,
                request.object_id.into(),
                request.edit_field_id,
                None,
            )
            .await?
        }
        EditFieldObjectKind::Schema => {
            Schema::update_from_edit_field(
                &ctx,
                request.object_id.into(),
                request.edit_field_id,
                None,
            )
            .await?
        }
        EditFieldObjectKind::SchemaUiMenu => {
            schema::UiMenu::update_from_edit_field(
                &ctx,
                request.object_id.into(),
                request.edit_field_id,
                None,
            )
            .await?
        }
        EditFieldObjectKind::SchemaVariant => {
            SchemaVariant::update_from_edit_field(
                &ctx,
                request.object_id.into(),
                request.edit_field_id,
                None,
            )
            .await?
        }
        EditFieldObjectKind::Socket => {
            Socket::update_from_edit_field(
                &ctx,
                request.object_id.into(),
                request.edit_field_id,
                None,
            )
            .await?
        }
    };

    txns.commit().await?;

    Ok(Json(RemoveFromEditFieldResponse { success: true }))
}