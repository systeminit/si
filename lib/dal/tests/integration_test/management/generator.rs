use std::collections::HashMap;

use dal::{diagram::view::View, Component, ComponentId, ComponentType, DalContext};
use dal_test::{helpers::create_component_for_default_schema_name_in_default_view, test};

#[test]
async fn calculates_top_and_center(ctx: &DalContext) {
    let default_view_id = View::get_id_for_default(ctx)
        .await
        .expect("get default view id");

    let mut center_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "Docker Image",
        "center lego",
    )
    .await
    .expect("make comp");
    let center_geo = center_lego
        .set_geometry(ctx, default_view_id, 0, 0, None, None)
        .await
        .expect("set center geo");

    let mut left_lego =
        create_component_for_default_schema_name_in_default_view(ctx, "Docker Image", "left lego")
            .await
            .expect("make comp");

    let left_geo = left_lego
        .set_geometry(ctx, default_view_id, -500, 0, None, None)
        .await
        .expect("set geo for left lego");

    let mut right_lego =
        create_component_for_default_schema_name_in_default_view(ctx, "Docker Image", "right lego")
            .await
            .expect("make comp");

    let right_geo = right_lego
        .set_geometry(ctx, default_view_id, 500, 0, None, None)
        .await
        .expect("set geo for left lego");

    let mut top_lego =
        create_component_for_default_schema_name_in_default_view(ctx, "Docker Image", "top lego")
            .await
            .expect("make comp");

    let top_geo = top_lego
        .set_geometry(ctx, default_view_id, 0, -500, None, None)
        .await
        .expect("set geo for left lego");

    let mut btm_lego =
        create_component_for_default_schema_name_in_default_view(ctx, "Docker Image", "btm lego")
            .await
            .expect("make comp");

    let btm_geo = btm_lego
        .set_geometry(ctx, default_view_id, 0, 500, None, None)
        .await
        .expect("set geo for left lego");

    let mut geometries = HashMap::new();
    for geo in [center_geo, left_geo, right_geo, btm_geo, top_geo] {
        geometries.insert(ComponentId::generate(), geo.into_raw());
    }

    let (origin_x, origin_y) =
        dal::management::generator::calculate_top_and_center(&geometries).await;

    assert_eq!(0, origin_x);
    assert_eq!(-1000, origin_y);

    Component::set_type_by_id(ctx, left_lego.id(), ComponentType::ConfigurationFrameDown)
        .await
        .expect("set type");
    Component::set_type_by_id(ctx, right_lego.id(), ComponentType::ConfigurationFrameDown)
        .await
        .expect("set type");
    let left_geo = left_lego
        .set_geometry(ctx, default_view_id, -500, 0, Some(750), Some(750))
        .await
        .expect("set geo for left again");
    let right_geo = right_lego
        .set_geometry(ctx, default_view_id, 500, 0, Some(750), Some(750))
        .await
        .expect("set geo for left again");

    let mut geometries = HashMap::new();
    for geo in [center_geo, left_geo, right_geo, btm_geo, top_geo] {
        geometries.insert(ComponentId::generate(), geo.into_raw());
    }

    let (origin_x, origin_y) =
        dal::management::generator::calculate_top_and_center(&geometries).await;

    assert_eq!(-50, origin_x);
    assert_eq!(-1000, origin_y);
}
