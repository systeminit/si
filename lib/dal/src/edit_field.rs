// //! [`EditFields`](EditField) are data bags that contain information for visualization and
// //! rendering for a given attribute at a specific [`AttributeContext`](crate::AttributeContext).
// //! Their [`EditFieldBaggage`] contains the [`AttributeValues`](crate::AttributeValue) to perform
// //! the actual CRUD-like operations for a given [`EditField`]. Essentially these fields are
// //! purely wrappers around [`AttributeValues`](crate::AttributeValue) that retain their lineage and
// //! ordering (important for [`Maps`](EditFieldDataType::Map) and
// //! [`Arrays`](EditFieldDataType::Array)).

pub mod widget;

// For diff reference--if you're reading this feel empowered to delete it!
//
// pub fn value_and_visibility_diff_option<Obj, Value: Eq + Serialize + ?Sized>(
//     visibility: &Visibility,
//     target_obj: Option<&Obj>,
//     target_fn: impl Fn(&Obj) -> Option<&Value> + Copy,
//     head_obj: Option<&Obj>,
//     change_set_obj: Option<&Obj>,
// ) -> EditFieldResult<(Option<serde_json::Value>, VisibilityDiff)> {
//     let target_value = target_obj.map(target_fn);
//     let head_value_option = head_obj.map(target_fn);
//     let change_set_value_option = change_set_obj.map(target_fn);
//     let visibility_diff = visibility_diff(
//         visibility,
//         target_value.as_ref(),
//         head_value_option.as_ref(),
//         change_set_value_option.as_ref(),
//     )?;
//     let mut value = None;
//     if let Some(target_value_real) = target_value {
//         value = Some(serde_json::to_value(target_value_real)?);
//     }
//     Ok((value, visibility_diff))
// }
//
// pub fn value_and_visibility_diff_json_option<Obj>(
//     visibility: &Visibility,
//     target_obj: Option<&Obj>,
//     target_fn: impl Fn(&Obj) -> Option<&serde_json::Value> + Copy,
//     head_obj: Option<&Obj>,
//     change_set_obj: Option<&Obj>,
// ) -> EditFieldResult<(Option<serde_json::Value>, VisibilityDiff)> {
//     let target_value = target_obj.map(target_fn);
//     let head_value_option = head_obj.map(target_fn);
//     let change_set_value_option = change_set_obj.map(target_fn);
//     let visibility_diff = visibility_diff(
//         visibility,
//         target_value.as_ref(),
//         head_value_option.as_ref(),
//         change_set_value_option.as_ref(),
//     )?;
//     let mut value = None;
//     if let Some(target_value_real) = target_value {
//         value = target_value_real.cloned();
//     }
//     Ok((value, visibility_diff))
// }
//
// pub fn value_and_visibility_diff<Obj, Value: Eq + Serialize + ?Sized>(
//     visibility: &Visibility,
//     target_obj: Option<&Obj>,
//     target_fn: impl Fn(&Obj) -> &Value + Copy,
//     head_obj: Option<&Obj>,
//     change_set_obj: Option<&Obj>,
// ) -> EditFieldResult<(Option<serde_json::Value>, VisibilityDiff)> {
//     let target_value = target_obj.map(target_fn);
//     let head_value_option = head_obj.map(target_fn);
//     let change_set_value_option = change_set_obj.map(target_fn);
//     let visibility_diff = visibility_diff(
//         visibility,
//         target_value,
//         head_value_option,
//         change_set_value_option,
//     )?;
//     let mut value = None;
//     if let Some(target_value_real) = target_value {
//         value = Some(serde_json::to_value(target_value_real)?);
//     }
//     Ok((value, visibility_diff))
// }
//
// pub fn value_and_visibility_diff_copy<Obj, Value: Eq + Serialize>(
//     visibility: &Visibility,
//     target_obj: Option<&Obj>,
//     target_fn: impl Fn(&Obj) -> Value + Copy,
//     head_obj: Option<&Obj>,
//     change_set_obj: Option<&Obj>,
// ) -> EditFieldResult<(Option<serde_json::Value>, VisibilityDiff)> {
//     let target_value = target_obj.map(target_fn);
//     let head_value_option = head_obj.map(target_fn);
//     let change_set_value_option = change_set_obj.map(target_fn);
//     let visibility_diff = visibility_diff(
//         visibility,
//         target_value.as_ref(),
//         head_value_option.as_ref(),
//         change_set_value_option.as_ref(),
//     )?;
//     let mut value = None;
//     if let Some(target_value_real) = target_value {
//         value = Some(serde_json::to_value(target_value_real)?);
//     }
//     Ok((value, visibility_diff))
// }
//
// fn visibility_diff<Value: Eq + Serialize + ?Sized>(
//     visibility: &Visibility,
//     target_value_option: Option<&Value>,
//     head_value_option: Option<&Value>,
//     change_set_value_option: Option<&Value>,
// ) -> EditFieldResult<VisibilityDiff> {
//     let mut visibility_diff = VisibilityDiff::None;
//     if visibility.in_change_set() {
//         visibility_diff = match (target_value_option, head_value_option) {
//             (Some(target_value), Some(head_value)) => {
//                 if target_value != head_value {
//                     VisibilityDiff::Head(Some(serde_json::to_value(head_value)?))
//                 } else {
//                     VisibilityDiff::None
//                 }
//             }
//             (Some(_), None) => VisibilityDiff::Head(None),
//             (None, Some(head_value)) => {
//                 VisibilityDiff::Head(Some(serde_json::to_value(head_value)?))
//             }
//             (None, None) => VisibilityDiff::None,
//         };
//     }
//     Ok(visibility_diff)
// }
