#[remain::sorted]
#[derive(
    Debug,
    Copy,
    Clone,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Default,
    strum::Display,
    strum::EnumString,
    Hash,
)]
/// The build priority for MVs is:
///   * List
///   * Detail
///
/// The order of the priorities is determined by the descriminant, largest to smallest.
pub enum BuildPriority {
    #[default]
    Detail = 5,
    List = 10,
}
