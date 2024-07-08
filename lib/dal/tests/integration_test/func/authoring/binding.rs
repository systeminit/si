mod action;
mod attribute;
mod authentication;

// #[test]
// #[ignore]
// async fn get_bindings_for_latest_schema_variants(ctx: &mut DalContext) {
//     let func_name = "test:createActionStarfield".to_string();

//     let func_id = Func::find_id_by_name(ctx, func_name)
//         .await
//         .expect("found func")
//         .expect("is some");

//     let mut bindings = FuncBinding::for_func_id(ctx, func_id)
//         .await
//         .expect("found func bindings");
//     dbg!(&bindings);
//     assert_eq!(bindings.len(), 1);

//     let binding = bindings.pop().expect("has a binding");

//     let old_schema_variant_id = binding.get_schema_variant().expect("has a schema variant");

//     // this schema variant is locked
//     let old_schema_variant = SchemaVariant::get_by_id(ctx, old_schema_variant_id)
//         .await
//         .expect("has a schema variant");

//     //this one is locked
//     assert!(old_schema_variant.is_locked());

//     let unlocked_binding = FuncBinding::get_bindings_for_unlocked_schema_variants(ctx, func_id)
//         .await
//         .expect("got latest unlocked");
//     // no unlocked bindings currently
//     assert!(unlocked_binding.is_empty());

//     // manually unlock the sv
//     let new_sv = VariantAuthoringClient::create_unlocked_variant_copy(ctx, old_schema_variant_id)
//         .await
//         .expect("created unlocked copy");

//     // new sv should have old funcs attached?!
//     let new_bindings = FuncBinding::for_func_id(ctx, func_id)
//         .await
//         .expect("has bindings");

//     assert_eq!(
//         2,                  // expected
//         new_bindings.len(), // actual
//     );

//     for binding in new_bindings {
//         let sv = SchemaVariant::get_by_id(ctx, binding.get_schema_variant().expect("has sv"))
//             .await
//             .expect("has sv");
//         dbg!(sv);
//     }

//     // now we should have 1 unlocked func binding
//     let mut latest_sv = FuncBinding::get_bindings_for_unlocked_schema_variants(ctx, func_id)
//         .await
//         .expect("got latest for default");

//     assert_eq!(1, latest_sv.len());

//     let latest_sv_from_binding = latest_sv
//         .pop()
//         .expect("has one")
//         .get_schema_variant()
//         .expect("has sv");
//     assert_eq!(latest_sv_from_binding, new_sv.id);

//     let latest_unlocked = FuncBinding::get_bindings_for_unlocked_schema_variants(ctx, func_id)
//         .await
//         .expect("got latest unlocked");

//     // should have one latest unlocked now!
//     assert_eq!(1, latest_unlocked.len());

//     // now create a copy of the func (unlock it!)

//     let new_func = FuncAuthoringClient::create_unlocked_func_copy(ctx, func_id, None)
//         .await
//         .expect("can create unlocked copy");
//     let new_func_id = new_func.id;

//     // get the bindings and make sure everything looks good
//     let mut latest_sv = FuncBinding::get_bindings_for_latest_schema_variants(ctx, new_func_id)
//         .await
//         .expect("got latest for default");

//     dbg!(&latest_sv);
//     assert_eq!(1, latest_sv.len());

//     // latest sv should be the new one!
//     let sv_id = latest_sv
//         .pop()
//         .expect("has one func")
//         .get_schema_variant()
//         .expect("has a schema variant");

//     assert_eq!(new_sv.id, sv_id);

//     // old func should have no unlocked variants
//     let unlocked_binding = FuncBinding::get_bindings_for_unlocked_schema_variants(ctx, func_id)
//         .await
//         .expect("got latest unlocked");
//     // no unlocked bindings currently
//     assert!(unlocked_binding.is_empty());
// }
