use dal::{DalContext, Func, FuncBackendKind, FuncBackendResponseType, FuncBinding, StandardModel};
use dal_test::test;
use ulid::Ulid;

#[test]
async fn run(ctx: &DalContext) {
    let mut func = Func::new(
        ctx,
        "reconciliation",
        FuncBackendKind::JsReconciliation,
        FuncBackendResponseType::Reconciliation,
    )
    .await
    .expect("cannot create func");
    func.set_handler(ctx, Some("reconciliation"))
        .await
        .expect("unable to set func handler");
    func.set_code_plaintext(
        ctx,
        Some(
            "function reconciliation(arg: Input) {
    // Map of attribute value ids of the domain sub-tree to the new value (from the resource)
    const updates = {};

    // Set of action names to be executed to update the resource based on the new domain
    let actions = new Set();

    const mapped = { '/root/domain/value': 'patch_value' };

    for (const [key, value] of Object.entries(mapped)) {
      const diff = arg[key];
      if (diff === undefined) continue;

      // Updates domain to fit the new resource
      updates[diff.domain.id] = diff.normalizedResource;

      // Provides hot patch
      actions.add(value);

      delete arg[key];
    }

    // Everything else can't be hot-patched, so let's delete + create
    for (const value of Object.values(arg)) {
      // Updates domain to fit the new resource
      updates[value.domain.id] = value.normalizedResource;

      // Overrides hot-patch actions as this will fix everything, it's just expensive
      actions = ['delete', 'create'];
    }

    return {
      updates,
      actions,
    };
}",
        ),
    )
    .await
    .expect("unable to set func code plaintext");
    let (region_ulid, value_ulid) = (Ulid::new(), Ulid::new());

    let (_, func_binding_return_value) = FuncBinding::create_and_execute(
        ctx,
        serde_json::json!({
            "/root/domain/region": {
                "domain": { "id": region_ulid.to_string(), "value": "us-east-2" },
                "resource": "us-east-1",
                "normalizedResource": "us-east-1",
            },
            "/root/domain/value": {
                "domain": { "id": value_ulid.to_string(), "value": 1 },
                "resource": 2,
                "normalizedResource": 2,
            },
        }),
        *func.id(),
        vec![],
    )
    .await
    .expect("failed to execute func binding");

    assert_eq!(
        func_binding_return_value.value(),
        Some(&serde_json::json!({
            "updates": { region_ulid.to_string(): "us-east-1", value_ulid.to_string(): 2 },
            "actions": ["delete", "create"],
        }))
    );
}
