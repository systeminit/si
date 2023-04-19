use dal::{
    standard_model, ChangeSet, ChangeSetPk, DalContext, Func, FuncBackendKind, Prop, PropId,
    PropKind, Schema, SchemaVariant, SchemaVariantId, StandardModel,
};
use dal_test::{
    test,
    test_harness::{create_func, create_schema, create_schema_variant, create_visibility_head},
};
use itertools::Itertools;

#[test]
async fn get_by_pk(ctx: &DalContext) {
    let schema = create_schema(ctx).await;
    let retrieved: Schema = standard_model::get_by_pk(ctx, "schemas", schema.pk())
        .await
        .expect("cannot get schema by pk");

    assert_eq!(schema, retrieved);
}

#[test]
async fn get_by_id(ctx: &mut DalContext) {
    let schema = create_schema(ctx).await;
    let head_visibility = create_visibility_head();
    let head_ctx = ctx.clone_with_new_visibility(head_visibility);

    let no_head: Option<Schema> = standard_model::get_by_id(&head_ctx, "schemas", schema.id())
        .await
        .expect("could not get schema by id");

    assert!(no_head.is_none(), "head object exists when it should not");

    let for_change_set: Schema = standard_model::get_by_id(ctx, "schemas", schema.id())
        .await
        .expect("could not get schema by id")
        .expect("change set object should exist but it does not");
    assert_eq!(&for_change_set.id(), &schema.id());
    assert_eq!(
        &for_change_set.visibility().change_set_pk,
        &schema.visibility().change_set_pk
    );

    let mut change_set = ChangeSet::get_by_pk(ctx, &ctx.visibility().change_set_pk)
        .await
        .unwrap()
        .unwrap();

    change_set
        .apply(ctx)
        .await
        .expect("cannot apply change set");

    let for_head: Schema = standard_model::get_by_id(&head_ctx, "schemas", schema.id())
        .await
        .expect("could not get schema by id")
        .expect("change set object should exist but it does not");
    assert_ne!(&for_head.pk(), &for_change_set.pk());
    assert_eq!(&for_head.id(), &for_change_set.id());
    assert_eq!(&for_head.visibility().change_set_pk, &ChangeSetPk::NONE,);
}

#[test]
async fn list(ctx: &DalContext) {
    let first_schema = create_schema(ctx).await;
    let second_schema = create_schema(ctx).await;
    let third_schema = create_schema(ctx).await;

    let head_visibility = create_visibility_head();
    let head_ctx = ctx.clone_with_new_visibility(head_visibility);

    let at_head: Vec<Schema> = standard_model::list(&head_ctx, "schemas")
        .await
        .expect("could not get schema by id");
    assert!(
        !at_head.iter().any(
            |el| vec![first_schema.id(), second_schema.id(), third_schema.id()].contains(&el.id())
        ),
        "schemas are in the set"
    );

    let at_change_set: Vec<Schema> = standard_model::list(ctx, "schemas")
        .await
        .expect("could not list schema");
    assert!(
        at_change_set.iter().any(|el| vec![
            first_schema.id(),
            second_schema.id(),
            third_schema.id()
        ]
        .contains(&el.id())),
        "schemas aren't in the set"
    );
}

#[test]
async fn update(ctx: &mut DalContext) {
    let schema = create_schema(ctx).await;
    let _updated_at = standard_model::update(
        ctx,
        "schemas",
        "name",
        schema.id(),
        &"funtime",
        standard_model::TypeHint::Text,
    )
    .await
    .expect("cannot update field");
}

#[test]
async fn delete(ctx: &DalContext) {
    let schema = create_schema(ctx).await;
    let _updated_at = standard_model::delete_by_pk(ctx, "schemas", schema.pk())
        .await
        .expect("cannot delete field");

    let soft_deleted: Schema = standard_model::get_by_pk(ctx, "schemas", schema.pk())
        .await
        .expect("cannot get schema");

    assert!(
        soft_deleted.visibility().deleted_at.is_some(),
        "should be deleted"
    );
}

#[test]
async fn hard_delete(ctx: &DalContext) {
    let schema = create_schema(ctx).await;
    let s: Schema = standard_model::get_by_pk(ctx, "schemas", schema.pk())
        .await
        .expect("cannot get schema");
    assert_eq!(s, schema);

    let hard_deleted = s.hard_delete(ctx).await.expect("could not hard delete");
    assert_eq!(schema.pk(), hard_deleted.pk());

    assert!(
        standard_model::get_by_pk::<_, Schema>(ctx, "schemas", hard_deleted.pk(),)
            .await
            .is_err()
    );
}

#[test]
async fn undelete(ctx: &DalContext) {
    let schema = create_schema(ctx).await;

    let _updated_at = standard_model::delete_by_pk(ctx, "schemas", schema.pk())
        .await
        .expect("cannot delete field");

    let soft_deleted: Schema = standard_model::get_by_pk(ctx, "schemas", schema.pk())
        .await
        .expect("cannot get schema");

    assert!(
        soft_deleted.visibility().deleted_at.is_some(),
        "should be deleted"
    );

    let _updated_at = standard_model::undelete(ctx, "schemas", schema.pk())
        .await
        .expect("cannot delete field");

    let soft_undeleted: Schema = standard_model::get_by_pk(ctx, "schemas", schema.pk())
        .await
        .expect("cannot get schemas");

    assert!(
        soft_undeleted.visibility().deleted_at.is_none(),
        "should be no longer deleted"
    );
}

#[test]
async fn set_belongs_to(ctx: &DalContext) {
    let schema = create_schema(ctx).await;
    let schema_variant = create_schema_variant(ctx, *schema.id()).await;

    let second_schema = create_schema(ctx).await;

    // Schema variant creation already sets belongs to
    let found_schema = schema_variant
        .schema(ctx)
        .await
        .expect("cannot retrieve schema from variant")
        .expect("cannot find schema from variant");
    assert_eq!(found_schema, schema);

    // You can replace the existing belongs to relationship by calling it again with a new id
    standard_model::set_belongs_to(
        ctx,
        "schema_variant_belongs_to_schema",
        schema_variant.id(),
        second_schema.id(),
    )
    .await
    .expect("cannot update variant for schema");

    let found_schema = schema_variant
        .schema(ctx)
        .await
        .expect("cannot retrieve schema from variant")
        .expect("cannot find schema from variant");
    assert_eq!(found_schema, second_schema);
}

#[test]
async fn unset_belongs_to(ctx: &DalContext) {
    let schema = create_schema(ctx).await;
    let schema_variant = create_schema_variant(ctx, *schema.id()).await;

    // Schema variant creation already sets belongs to
    let found_schema = schema_variant
        .schema(ctx)
        .await
        .expect("cannot retrieve schema from variant")
        .expect("cannot find schema from variant");
    assert_eq!(found_schema, schema);

    standard_model::unset_belongs_to(ctx, "schema_variant_belongs_to_schema", schema_variant.id())
        .await
        .expect("cannot set variant for schema");
}

#[test]
async fn belongs_to(ctx: &mut DalContext) {
    let schema = create_schema(ctx).await;
    let schema_variant = create_schema_variant(ctx, *schema.id()).await;

    let visibility_head = create_visibility_head();
    let head_ctx = ctx.clone_with_new_visibility(visibility_head);
    let no_head: Option<Schema> = standard_model::belongs_to(
        &head_ctx,
        "schema_variant_belongs_to_schema",
        "schemas",
        schema_variant.id(),
    )
    .await
    .expect("cannot update variant for schema");
    assert!(no_head.is_none(), "head relationship should not exist");

    let has_change_set: Option<Schema> = standard_model::belongs_to(
        ctx,
        "schema_variant_belongs_to_schema",
        "schemas",
        schema_variant.id(),
    )
    .await
    .expect("cannot fetch variant for schema");
    assert!(
        has_change_set.is_some(),
        "change set relationship should exist"
    );

    let mut change_set = ChangeSet::get_by_pk(ctx, &ctx.visibility().change_set_pk)
        .await
        .unwrap()
        .unwrap();

    change_set
        .apply(ctx)
        .await
        .expect("cannot apply change set");

    let has_head: Option<Schema> = standard_model::belongs_to(
        &head_ctx,
        "schema_variant_belongs_to_schema",
        "schemas",
        schema_variant.id(),
    )
    .await
    .expect("cannot fetch variant for schema");
    assert!(has_head.is_some(), "head relationship should exist");

    standard_model::unset_belongs_to(
        &head_ctx,
        "schema_variant_belongs_to_schema",
        schema_variant.id(),
    )
    .await
    .expect("cannot unset variant for schema");
    let no_head: Option<Schema> = standard_model::belongs_to(
        &head_ctx,
        "schema_variant_belongs_to_schema",
        "schemas",
        schema_variant.id(),
    )
    .await
    .expect("cannot get variant from schema");
    assert!(
        no_head.is_none(),
        "head relationship should no longer exist"
    );
}

#[test]
async fn has_many(ctx: &DalContext) {
    let schema = create_schema(ctx).await;
    let schema_variant = create_schema_variant(ctx, *schema.id()).await;
    let second_schema_variant = create_schema_variant(ctx, *schema.id()).await;

    let visibility_head = create_visibility_head();
    let head_ctx = ctx.clone_with_new_visibility(visibility_head);
    let no_head: Vec<SchemaVariant> = standard_model::has_many(
        &head_ctx,
        "schema_variant_belongs_to_schema",
        "schema_variants",
        schema.id(),
    )
    .await
    .expect("cannot get variants from schemas");
    assert!(no_head.is_empty(), "head relationship should not exist");

    let schema_variants: Vec<SchemaVariant> = standard_model::has_many(
        ctx,
        "schema_variant_belongs_to_schema",
        "schema_variants",
        schema.id(),
    )
    .await
    .expect("cannot get variants from schemas");
    assert_eq!(schema_variants.len(), 2);
    assert_eq!(
        schema_variants
            .into_iter()
            .filter(|k| k == &schema_variant || k == &second_schema_variant)
            .count(),
        2
    );
}

#[test]
async fn associate_many_to_many(ctx: &DalContext) {
    let prop_one = Prop::new(ctx, "prop_one", PropKind::String, None)
        .await
        .expect("unable to create prop");
    let prop_two = Prop::new(ctx, "prop_two", PropKind::String, None)
        .await
        .expect("unable to create prop");

    let schema = create_schema(ctx).await;
    let schema_variant_one = create_schema_variant(ctx, *schema.id()).await;
    let schema_variant_two = create_schema_variant(ctx, *schema.id()).await;

    standard_model::associate_many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        prop_one.id(),
        schema_variant_one.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::associate_many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        prop_one.id(),
        schema_variant_two.id(),
    )
    .await
    .expect("cannot associate many to many");

    standard_model::associate_many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        prop_two.id(),
        schema_variant_one.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::associate_many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        prop_two.id(),
        schema_variant_two.id(),
    )
    .await
    .expect("cannot associate many to many");
}

#[test]
async fn disassociate_many_to_many(ctx: &DalContext) {
    let prop_one = Prop::new(ctx, "prop_one", PropKind::String, None)
        .await
        .expect("unable to create prop");

    let schema = create_schema(ctx).await;
    let schema_variant_one = create_schema_variant(ctx, *schema.id()).await;
    let schema_variant_two = create_schema_variant(ctx, *schema.id()).await;

    standard_model::associate_many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        prop_one.id(),
        schema_variant_one.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::associate_many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        prop_one.id(),
        schema_variant_two.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::disassociate_many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        prop_one.id(),
        schema_variant_two.id(),
    )
    .await
    .expect("cannot disassociate many to many");
}

#[test]
async fn disassociate_all_many_to_many(ctx: &DalContext) {
    let prop_one = Prop::new(ctx, "prop_one", PropKind::String, None)
        .await
        .expect("unable to create prop");

    let schema = create_schema(ctx).await;
    let schema_variant_one = create_schema_variant(ctx, *schema.id()).await;
    let schema_variant_two = create_schema_variant(ctx, *schema.id()).await;

    standard_model::associate_many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        prop_one.id(),
        schema_variant_one.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::associate_many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        prop_one.id(),
        schema_variant_two.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::disassociate_all_many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        prop_one.id(),
    )
    .await
    .expect("cannot disassociate many to many");
}

#[test]
async fn many_to_many(ctx: &DalContext) {
    let prop_one = Prop::new(ctx, "prop_one", PropKind::String, None)
        .await
        .expect("unable to create prop");
    let prop_two = Prop::new(ctx, "prop_two", PropKind::String, None)
        .await
        .expect("unable to create prop");

    let schema = create_schema(ctx).await;
    let schema_variant_one = create_schema_variant(ctx, *schema.id()).await;
    let schema_variant_two = create_schema_variant(ctx, *schema.id()).await;

    standard_model::associate_many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        prop_one.id(),
        schema_variant_one.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::associate_many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        prop_one.id(),
        schema_variant_two.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::associate_many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        prop_two.id(),
        schema_variant_two.id(),
    )
    .await
    .expect("cannot associate many to many");

    let right_object_id: Option<&SchemaVariantId> = None;
    let left_object_id: Option<&PropId> = None;
    let prop_variants: Vec<SchemaVariant> = standard_model::many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        "props",
        "schema_variants",
        Some(prop_one.id()),
        right_object_id,
    )
    .await
    .expect("cannot get list of users for group");
    assert_eq!(prop_variants.len(), 2);
    assert_eq!(
        prop_variants
            .into_iter()
            .filter(|v| v == &schema_variant_one || v == &schema_variant_two)
            .count(),
        2
    );

    standard_model::disassociate_many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        prop_two.id(),
        schema_variant_two.id(),
    )
    .await
    .expect("cannot disassociate many to many");

    let variant_two_props: Vec<Prop> = standard_model::many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        "props",
        "schema_variants",
        left_object_id,
        Some(schema_variant_two.id()),
    )
    .await
    .expect("cannot get list of props for variant");
    assert_eq!(
        variant_two_props
            .into_iter()
            .filter(|p| p == &prop_one)
            .count(),
        1
    );

    standard_model::associate_many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        prop_two.id(),
        schema_variant_two.id(),
    )
    .await
    .expect("cannot associate many to many");

    let variant_two_props: Vec<Prop> = standard_model::many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        "props",
        "schema_variants",
        left_object_id,
        Some(schema_variant_two.id()),
    )
    .await
    .expect("cannot get list of props for variant");
    assert_eq!(variant_two_props.len(), 3);
    assert_eq!(
        variant_two_props
            .into_iter()
            .filter(|p| p == &prop_one || p == &prop_two)
            .count(),
        2
    );

    standard_model::disassociate_all_many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        prop_two.id(),
    )
    .await
    .expect("cannot disassociate many to many");

    let variant_two_props: Vec<Prop> = standard_model::many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        "props",
        "schema_variants",
        left_object_id,
        Some(schema_variant_two.id()),
    )
    .await
    .expect("cannot get list of groups for user");
    assert_eq!(
        variant_two_props
            .into_iter()
            .filter(|p| p == &prop_one)
            .count(),
        1
    );
}

#[test]
async fn associate_many_to_many_no_repeat_entries(ctx: &DalContext) {
    let prop_one = Prop::new(ctx, "prop_one", PropKind::String, None)
        .await
        .expect("unable to create prop");

    let schema = create_schema(ctx).await;
    let schema_variant_one = create_schema_variant(ctx, *schema.id()).await;

    standard_model::associate_many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        prop_one.id(),
        schema_variant_one.id(),
    )
    .await
    .expect("cannot associate many to many");
    let result = standard_model::associate_many_to_many(
        ctx,
        "prop_many_to_many_schema_variants",
        prop_one.id(),
        schema_variant_one.id(),
    )
    .await;
    assert!(result.is_err(), "should error");
}

#[test]
async fn find_by_attr(ctx: &mut DalContext) {
    let schema_one = create_schema(ctx).await;
    let schema_variant_one = create_schema_variant(ctx, *schema_one.id()).await;

    let result: Vec<Schema> =
        standard_model::find_by_attr(ctx, "schemas", "name", &schema_one.name().to_string())
            .await
            .expect("cannot find the object by name");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], schema_one);

    let schema_two = create_schema(ctx).await;
    let mut schema_variant_two = create_schema_variant(ctx, *schema_two.id()).await;
    schema_variant_two
        .set_name(ctx, schema_variant_one.name())
        .await
        .expect("unable to set name");

    let result: Vec<SchemaVariant> = standard_model::find_by_attr(
        ctx,
        "schema_variants",
        "name",
        &schema_variant_one.name().to_string(),
    )
    .await
    .expect("cannot find the object by name");
    assert_eq!(result.len(), 2);
    assert_eq!(
        result
            .into_iter()
            .filter(|r| r == &schema_variant_one || r == &schema_variant_two)
            .count(),
        2
    );
}

#[test]
async fn find_by_attr_in(ctx: &mut DalContext) {
    // There are some functions in here already but we don't want to rely on
    // them existing for the test to pass
    let first_result: Vec<Func> = standard_model::find_by_attr_in(
        ctx,
        "funcs",
        "backend_kind",
        &[&"JsWorkflow".to_string(), &"JsAttribute".to_string()],
    )
    .await
    .expect("cannot find objects by backend_kind in slice");

    let mut func_one = create_func(ctx).await;
    func_one
        .set_backend_kind(ctx, FuncBackendKind::JsWorkflow)
        .await
        .expect("cannot set func backend kind");

    let mut func_two = create_func(ctx).await;
    func_two
        .set_backend_kind(ctx, FuncBackendKind::JsAttribute)
        .await
        .expect("cannot set func backend kind");

    let result: Vec<Func> = standard_model::find_by_attr_in(
        ctx,
        "funcs",
        "backend_kind",
        &[
            &FuncBackendKind::JsWorkflow.as_ref().to_string(),
            &FuncBackendKind::JsAttribute.as_ref().to_string(),
        ],
    )
    .await
    .expect("cannot find objects by backend_kind in slice");

    assert_eq!(2, result.len() - first_result.len());

    assert_eq!(
        Some(&func_one),
        result
            .iter()
            .filter(|&f| f.id() == func_one.id())
            .at_most_one()
            .expect("could not find at most one func")
    );

    assert_eq!(
        Some(&func_two),
        result
            .iter()
            .filter(|&f| f.id() == func_two.id())
            .at_most_one()
            .expect("could not find at most one func")
    );
}

#[test]
async fn find_by_attr_not_in(ctx: &mut DalContext) {
    let func_one = create_func(ctx).await;
    let func_two = create_func(ctx).await;

    let func_one_name = func_one.name().to_string();
    let func_two_name = func_two.name().to_string();

    let result: Vec<Func> = standard_model::find_by_attr_not_in(
        ctx,
        "funcs",
        "name",
        &[&func_one_name, &func_two_name],
    )
    .await
    .expect("cannot find objects by backend_kind in slice");

    assert_eq!(
        None,
        result
            .iter()
            .filter(|&f| f.id() == func_one.id())
            .at_most_one()
            .expect("could not find at most one func")
    );

    assert_eq!(
        None,
        result
            .iter()
            .filter(|&f| f.id() == func_two.id())
            .at_most_one()
            .expect("could not find at most one func")
    );
}
