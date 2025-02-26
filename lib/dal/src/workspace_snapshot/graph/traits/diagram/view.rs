use anyhow::Result;
use si_id::ComponentId;

use crate::diagram::view::ViewId;

pub trait ViewExt {
    fn view_remove(&mut self, view_id: ViewId) -> Result<()>;

    fn list_for_component_id(&self, component_id: ComponentId) -> Result<Vec<ViewId>>;
}
