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

    assert_eq!(11, modules.len());

    let expected_installed_module_names = vec![
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
