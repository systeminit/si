use dal::{DalContext, Secret};
use dal_test::{test, WorkspaceSignup};

#[test]
async fn list_ids_by_key_bench(ctx: &mut DalContext, nw: &WorkspaceSignup) {
    let secrets_count = 1000;
    let key_pair_pk = nw.key_pair.pk();

    // Populate the graph with many secrets.
    let secret_creation_instant = tokio::time::Instant::now();
    for count in 0..secrets_count {
        if count % 100 == 0 {
            println!(
                "creating secret {count} of {secrets_count} ({:?})",
                secret_creation_instant.elapsed()
            );
        }
        Secret::new(
            ctx,
            count.to_string(),
            count.to_string(),
            None,
            &[],
            key_pair_pk,
            Default::default(),
            Default::default(),
        )
        .await
        .expect("could not create secret");
    }
    println!(
        "creating {secrets_count} secrets took: {:?}",
        secret_creation_instant.elapsed()
    );

    let ids_by_key = Secret::list_ids_by_key(ctx)
        .await
        .expect("could not list ids by key");

    // Check the number of keys in the map. Their length should be the same as the number of secrets
    // created.
    assert_eq!(
        secrets_count,           // expected
        ids_by_key.keys().len(), // actual
    );
}
