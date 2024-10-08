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
    V2(InnerType, InnerType),
    #[si_versioned_node_weight(current)]
    V3 {
        inner: InnerType,
    },
    #[si_versioned_node_weight(current)]
    V4,
    V5,
    V6(InnerType),
    V7(InnerType, InnerType),
    V8 {
        inner: InnerType,
    },
}

fn main() {}
