use dal::diagram::node::{DiagramSocketDirection, DiagramSocketView};
use dal::{DalContext, Schema, SchemaVariant};
use dal_test::test;
use std::collections::HashSet;

#[test]
async fn socket_views(ctx: &DalContext) {
    // Get the source schema variant id.
    let docker_image_schema = Schema::find_by_name(ctx, "Docker Image")
        .await
        .expect("could not perform find by name")
        .expect("no schema found");
    let mut docker_image_schema_variants =
        SchemaVariant::list_for_schema(ctx, docker_image_schema.id())
            .await
            .expect("could not list schema variants for schema");
    let docker_image_schema_variant = docker_image_schema_variants
        .pop()
        .expect("schema variants are empty");
    let docker_image_schema_variant_id = docker_image_schema_variant.id();

    // Get the destination schema variant id.
    let butane_schema = Schema::find_by_name(ctx, "Butane")
        .await
        .expect("could not perform find by name")
        .expect("no schema found");
    let mut butane_schema_variants = SchemaVariant::list_for_schema(ctx, butane_schema.id())
        .await
        .expect("could not list schema variants for schema");
    let butane_schema_variant = butane_schema_variants
        .pop()
        .expect("schema variants are empty");
    let butane_schema_variant_id = butane_schema_variant.id();

    // List and check socket views.
    let docker_image_sockets = DiagramSocketView::list(ctx, docker_image_schema_variant_id)
        .await
        .expect("could not get socket views")
        .iter()
        .map(|sv| (sv.label.to_owned(), sv.direction))
        .collect::<HashSet<(String, DiagramSocketDirection)>>();
    let expected_docker_image_sockets = HashSet::from([
        (
            "Docker Hub Credential".to_string(),
            DiagramSocketDirection::Input,
        ),
        ("Frame".to_string(), DiagramSocketDirection::Output),
        ("Exposed Ports".to_string(), DiagramSocketDirection::Output),
        (
            "Container Image".to_string(),
            DiagramSocketDirection::Output,
        ),
        ("Frame".to_string(), DiagramSocketDirection::Input),
    ]);
    let butane_sockets = DiagramSocketView::list(ctx, butane_schema_variant_id)
        .await
        .expect("could not get socket views")
        .iter()
        .map(|sv| (sv.label.to_owned(), sv.direction))
        .collect::<HashSet<(String, DiagramSocketDirection)>>();
    let expected_butane_sockets = HashSet::from([
        ("Frame".to_string(), DiagramSocketDirection::Output),
        ("User Data".to_string(), DiagramSocketDirection::Output),
        ("Container Image".to_string(), DiagramSocketDirection::Input),
        ("Frame".to_string(), DiagramSocketDirection::Input),
    ]);
    assert_eq!(
        expected_docker_image_sockets, // expected
        docker_image_sockets           // actual
    );
    assert_eq!(
        expected_butane_sockets, // expected
        butane_sockets           // actual
    );
}
