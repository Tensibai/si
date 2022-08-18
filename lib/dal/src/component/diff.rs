//! This module contains [`ComponentDiff`].

use serde::{Deserialize, Serialize};

use crate::component::ComponentResult;
use crate::{
    AttributeReadContext, CodeLanguage, CodeView, Component, ComponentError, ComponentId,
    ComponentView, DalContext, StandardModel,
};

const NEWLINE: &str = "\n";

// NOTE(nick): while the destination is the browser, we may want to consider platform-specific
// newline characters.
// #[cfg(target_os != "windows")]
// const NEWLINE: &str = "\n";
// #[cfg(target_os = "windows")]
// const NEWLINE: &str = "\r\n";

/// Contains the "diffs" for a given [`Component`](crate::Component). Generated by
/// [`Self::new()`].
#[derive(Deserialize, Serialize, Debug)]
pub struct ComponentDiff {
    /// The [`Component's`](crate::Component) [`CodeView`](crate::code_view::CodeView) found in the
    /// current [`Visibility`](crate::Visibility).
    pub current: CodeView,
    /// The "diff(s)" between [`Component`](crate::Component)'s
    /// [`CodeViews`](crate::code_view::CodeView) found on _head_ and found in the current
    /// [`Visibility`](crate::Visibility).
    ///
    /// This will be empty if the [`Component`](crate::Component) has been newly added.
    pub diffs: Vec<CodeView>,
}

impl ComponentDiff {
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        head_ctx: &DalContext<'_, '_>,
        component_id: ComponentId,
    ) -> ComponentResult<Self> {
        if ctx.visibility().is_head() || !head_ctx.visibility().is_head() {
            return Err(ComponentError::InvalidContextForDiff);
        }

        // FIXME(nick): this is inefficient and should be replaced with a single query that returns
        // the three IDs. The query could leverage the "components_with_attributes" table.
        let component = Component::get_by_id(ctx, &component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))?;
        let schema_variant = component
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;
        let schema = component
            .schema(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let root_prop = schema_variant.root_prop(ctx).await?;

        let component_view_context = AttributeReadContext {
            prop_id: Some(*root_prop.id()),
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            component_id: Some(component_id),
            ..AttributeReadContext::default()
        };

        // TODO(nick): perhaps, we can serialize the value into other kinds of structure in the future.
        let curr_component_view = ComponentView::for_context(ctx, component_view_context).await?;
        let curr_json = serde_json::to_string_pretty(&curr_component_view.properties)?;

        // Find the "diffs" given the head dal context only if the component exists on head.
        let diffs: Vec<CodeView> = if Component::get_by_id(head_ctx, &component_id)
            .await?
            .is_some()
        {
            let prev_component_view =
                ComponentView::for_context(head_ctx, component_view_context).await?;
            let prev_json = serde_json::to_string_pretty(&prev_component_view.properties)?;

            let mut lines = Vec::new();
            for diff_object in diff::lines(&prev_json, &curr_json) {
                let line = match diff_object {
                    diff::Result::Left(left) => format!("-{}", left),
                    diff::Result::Both(unchanged, _) => format!(" {}", unchanged),
                    diff::Result::Right(right) => format!("+{}", right),
                };
                lines.push(line);
            }

            // FIXME(nick): generate multiple code views if there are multiple code views.
            let diff = CodeView::new(CodeLanguage::Diff, Some(lines.join(NEWLINE)));
            vec![diff]
        } else {
            vec![]
        };

        Ok(Self {
            current: CodeView::new(CodeLanguage::Json, Some(curr_json)),
            diffs,
        })
    }
}