use dal::module::Module;
use dal::DalContext;
use dal_test::test;

#[test]
async fn list_pkgs(ctx: &DalContext) {
    let modules = Module::list_installed(ctx)
        .await
        .expect("unable to get installed modules");

    let mut module_names: Vec<String> = modules.iter().map(|m| m.name().to_string()).collect();
    module_names.sort();

    assert_eq!(14, modules.len());

    let expected_installed_module_names = vec![
        "BadValidations".to_string(),
        "ValidatedInput".to_string(),
        "ValidatedOutput".to_string(),
        "dummy-secret".to_string(),
        "fallout".to_string(),
        "katy perry".to_string(),
        "pet_shop".to_string(),
        "pirate".to_string(),
        "si-aws-ec2-2023-09-26-2".to_string(),
        "si-coreos-2023-09-13".to_string(),
        "si-docker-image-2023-09-13".to_string(),
        "si-intrinsic-funcs".to_string(),
        "starfield".to_string(),
        "swifty".to_string(),
    ];

    assert_eq!(expected_installed_module_names, module_names);
}

#[test]
async fn get_fallout_pkg(ctx: &DalContext) {
    let modules = Module::list_installed(ctx)
        .await
        .expect("unable to get installed modules");

    let mut filtered_modules: Vec<Module> = modules
        .into_iter()
        .filter(|m| m.name() == "fallout")
        .collect();

    assert_eq!(1, filtered_modules.len());

    if let Some(fallout_module) = filtered_modules.pop() {
        let associated_funcs = fallout_module
            .list_associated_funcs(ctx)
            .await
            .expect("Unable to get association funcs");
        let associated_schemas = fallout_module
            .list_associated_schemas(ctx)
            .await
            .expect("Unable to get association schemas");

        assert_eq!("fallout", fallout_module.name());
        assert_eq!("System Initiative", fallout_module.created_by_email());
        assert_eq!("2019-06-03", fallout_module.version());
        assert_eq!(3, associated_funcs.len());
        assert_eq!(1, associated_schemas.len());
    }
}
