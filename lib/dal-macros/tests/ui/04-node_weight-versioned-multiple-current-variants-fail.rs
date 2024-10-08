pub struct InnerType;

pub trait SiVersionedNodeWeight {
    type Inner;

    fn inner(&self) -> &Self::Inner;
    fn inner_mut(&mut self) -> &mut Self::Inner;
}

#[derive(dal_macros::SiVersionedNodeWeight)]
pub enum VersionedNodeWeight {
    #[si_versioned_node_weight(current)]
    V1(InnerType),
    #[si_versioned_node_weight(current)]
    V2(InnerType),
    V3(InnerType),
    V4(InnerType),
    #[si_versioned_node_weight(current)]
    V5(InnerType),
}

fn main() {}
