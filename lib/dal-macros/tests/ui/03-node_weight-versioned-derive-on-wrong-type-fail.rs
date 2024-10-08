pub struct InnerType;

pub trait SiVersionedNodeWeight {
    type Inner;

    fn inner(&self) -> &Self::Inner;
    fn inner_mut(&mut self) -> &mut Self::Inner;
}

#[derive(dal_macros::SiVersionedNodeWeight)]
pub struct VersionedNodeWeight {}

fn main() {}
