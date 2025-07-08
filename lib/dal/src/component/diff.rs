//! This module contains functionalk

use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    Component,
    ComponentError,
    ComponentId,
    DalContext,
    code_view::{
        CodeLanguage,
        CodeView,
    },
    component::{
        ComponentResult,
        properties::ComponentProperties,
    },
};

// NOTE(nick): this will not work on Windows.
const NEWLINE: &str = "\n";

/// Contains a rendered diff between the [`Component`](Self) on HEAD and in the current
/// [`ChangeSet`](crate::ChangeSet).
#[derive(Deserialize, Serialize, Debug)]
pub struct ComponentDiff {
    pub component_id: ComponentId,
    pub current: CodeView,
    pub diff: Option<CodeView>,
}

impl Component {
    /// Generates a [`ComponentDiff`] for a given [`ComponentId`](crate::Component).
    pub async fn get_diff(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<ComponentDiff> {
        let (curr_json, maybe_head_json_with_is_new_check) =
            Self::get_diff_inner(ctx, component_id).await?;
        let curr_json = serde_json::to_string_pretty(&curr_json)?;

        let (head_json, is_new_comp) =
            if let Some((head_json, is_new_comp)) = maybe_head_json_with_is_new_check {
                (serde_json::to_string_pretty(&head_json)?, is_new_comp)
            } else {
                return Ok(ComponentDiff {
                    component_id,
                    current: CodeView::assemble(CodeLanguage::Json, Some(curr_json), None, None),
                    diff: None,
                });
            };

        let mut lines = Vec::new();
        for diff_object in diff::lines(&head_json, &curr_json) {
            let line = match diff_object {
                diff::Result::Left(left) => format!("-{left}"),
                diff::Result::Both(unchanged, _) => format!(" {unchanged}"),
                diff::Result::Right(right) => format!("+{right}"),
            };
            if line != "-null" {
                lines.push(line);
            }
        }

        Ok(ComponentDiff {
            component_id,
            current: CodeView::assemble(
                CodeLanguage::Json,
                if is_new_comp { Some(curr_json) } else { None },
                None,
                None,
            ),
            diff: Some(CodeView::assemble(
                CodeLanguage::Diff,
                Some(lines.join(NEWLINE)),
                None,
                None,
            )),
        })
    }

    async fn get_diff_inner(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<(serde_json::Value, Option<(serde_json::Value, bool)>)> {
        let curr_json = match Self::view_by_id(ctx, component_id).await? {
            Some(view) => {
                let mut current_component_view = ComponentProperties::try_from(view)?;
                current_component_view.drop_private();
                serde_json::to_value(&current_component_view)?
            }
            None => serde_json::Value::Null,
        };

        // If we are on HEAD, then we have no diff to compute.
        let head_ctx = if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
            return Ok((curr_json, None));
        } else {
            ctx.clone_with_head().await?
        };

        let (head_json, is_new_component) = if Self::exists_by_id(&head_ctx, component_id).await? {
            match Self::view_by_id(&head_ctx, component_id).await? {
                Some(view) => {
                    let mut head_component_view = ComponentProperties::try_from(view)?;
                    head_component_view.drop_private();
                    (serde_json::to_value(&head_component_view)?, false)
                }
                None => (serde_json::Value::Null, false),
            }
        } else {
            (serde_json::Value::Null, true)
        };

        Ok((curr_json, Some((head_json, is_new_component))))
    }

    /// Returns the JSON representation for a given [`Component`](crate::Component).
    pub async fn get_json_representation(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<ComponentProperties> {
        Ok(match Self::view_by_id(ctx, component_id).await? {
            Some(view) => ComponentProperties::try_from(view)?,
            None => ComponentProperties::default(),
        })
    }

    /// Returns whether or not a component would have a diff when compared to head.
    pub async fn has_diff_from_head(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<bool> {
        // By definition there aren't changes between head and head
        if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
            return Ok(false);
        }

        // If component doesn't exist on head, then it is new and there is no diff
        if !Self::exists_on_head_by_id(ctx, component_id).await? {
            return Ok(false);
        }

        // Now we need serialized, textual versions of the component on this change set and on head
        // in order to calculate a textural diff
        //
        // NOTE(fnichol): this ported implementation is still way too expensive and relies on
        // `AttributeValue.view()` which uses `async_recursion`. This needs to change...

        let this_component_json_str = {
            let view = Self::view_by_id(ctx, component_id).await?.ok_or(
                ComponentError::AttributeValueView(
                    ctx.workspace_pk()?,
                    ctx.change_set_id(),
                    component_id,
                ),
            )?;
            let mut component_view = ComponentProperties::try_from(view)?;
            component_view.drop_private();

            serde_json::to_string_pretty(&component_view)?
        };

        let head_component_json_str = {
            let ctx = ctx.clone_with_head().await?;

            let view = Self::view_by_id(&ctx, component_id).await?.ok_or(
                ComponentError::AttributeValueView(
                    ctx.workspace_pk()?,
                    ctx.change_set_id(),
                    component_id,
                ),
            )?;
            let mut component_view = ComponentProperties::try_from(view)?;
            component_view.drop_private();

            serde_json::to_string_pretty(&component_view)?
        };

        // If the serialized strings differ, then there would be a diff
        Ok(this_component_json_str != head_component_json_str)
    }
}
