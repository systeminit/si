use dal::{DalContext, Secret};
use dal_test::{test, WorkspaceSignup};
use std::time::Duration;

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

    // Now that we have a graph with many secrets, let's run the function and cache the result.
    // Ensure that the result meets our expectations for wall clock time.
    let list_ids_by_key_instant = tokio::time::Instant::now();
    let ids_by_key = Secret::list_ids_by_key(ctx)
        .await
        .expect("could not list ids by key");
    let list_ids_by_key_instant_elapsed = list_ids_by_key_instant.elapsed();
    assert!(Duration::from_millis(10) > list_ids_by_key_instant_elapsed);
    println!(
        "list ids by key for {secrets_count} secrets took: {:?}",
        list_ids_by_key_instant_elapsed
    );

    // Check the number of keys in the map. Their length should be the same as the number of secrets
    // created.
    assert_eq!(
        secrets_count,           // expected
        ids_by_key.keys().len(), // actual
    );
}
