use base64::Engine;
use dal::{
    DalContext,
    Func,
    Schema,
    func::leaf::{
        LeafInputLocation,
        LeafKind,
    },
    schema::leaf::LeafPrototype,
};
use dal_test::{
    Result,
    test,
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn leaf_prototype_basic_test(ctx: &mut DalContext) -> Result<()> {
    let schema = Schema::get_by_name(ctx, "swifty").await?;

    let inputs = vec![
        LeafInputLocation::Domain,
        LeafInputLocation::DeletedAt,
        LeafInputLocation::Code,
        LeafInputLocation::Secrets,
        LeafInputLocation::Resource,
    ];

    let leaf_func_code = "async function main() {
                return { };
            }";

    let leaf_func = Func::new(
        ctx,
        "test:schemaLevelQualification",
        None::<String>,
        None::<String>,
        None::<String>,
        false,
        false,
        dal::FuncBackendKind::JsAttribute,
        dal::FuncBackendResponseType::Qualification,
        "main".into(),
        Some(base64::engine::general_purpose::STANDARD_NO_PAD.encode(leaf_func_code)),
        false,
    )
    .await?;

    let prototype = LeafPrototype::new(
        ctx,
        schema.id(),
        LeafKind::Qualification,
        inputs.clone(),
        leaf_func.id,
    )
    .await?;

    let fetched_prototype = LeafPrototype::get_by_id(ctx, prototype.id()).await?;

    assert_eq!(prototype, fetched_prototype);

    let leaf_inputs: Vec<_> = fetched_prototype.leaf_inputs().collect();

    assert_eq!(inputs, leaf_inputs);

    let func_id = LeafPrototype::func_id(ctx, prototype.id())
        .await?
        .expect("should get a func id");

    assert_eq!(leaf_func.id, func_id);

    Ok(())
}
