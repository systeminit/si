use std::collections::HashMap;

use si_frontend_types::RawGeometry;
use si_id::{ComponentId, ViewId};

use crate::{Component, ComponentType, DalContext};

use super::{ManagementOperations, ManagementResult};

pub async fn generate_template(
    ctx: &DalContext,
    view_id: ViewId,
    component_ids: &[ComponentId],
) -> ManagementResult<ManagementOperations> {
    // gather up the geometries of the input components
    let mut components = HashMap::new();
    let mut geometries = HashMap::new();

    for &component_id in component_ids {
        let component = Component::get_by_id(ctx, component_id).await?;
        let mut geometry = component.geometry(ctx, view_id).await?.into_raw();

        // We want to be sure that frames, and only frames, always have a height/width, while
        // components never have one.
        match component.get_type(ctx).await? {
            ComponentType::AggregationFrame
            | ComponentType::ConfigurationFrameDown
            | ComponentType::ConfigurationFrameUp => {
                if geometry.width.zip(geometry.height).is_none() {
                    geometry.width = Some(500);
                    geometry.height = Some(500);
                }

                // Adjust the x coordinate of a frame, since it is actually the center of the
                // frame, not the top left corner
                geometry.x = geometry.x - (geometry.width.unwrap_or(0) / 2);
            }
            ComponentType::Component => {
                if geometry.width.is_some() || geometry.height.is_some() {
                    geometry.width = None;
                    geometry.height = None;
                }
            }
        }

        components.insert(component_id, component);
        geometries.insert(component_id, geometry);
    }

    todo!()
}

fn calculate_top_and_center(geometries: HashMap<ComponentId, RawGeometry>) -> (isize, isize) {
    let mut topmost: Option<isize> = None;
    let mut leftmost: Option<isize> = None;
    let mut rightmost: Option<isize> = None;

    for geometry in geometries.values() {
        let current_topmost = topmost.unwrap_or(geometry.y);
        if geometry.y <= current_topmost {
            topmost.replace(geometry.y);
        }

        let current_leftmost = leftmost.unwrap_or(geometry.x);
        if geometry.x <= current_leftmost {
            leftmost.replace(geometry.x);
        }
    }

    todo!()
}
