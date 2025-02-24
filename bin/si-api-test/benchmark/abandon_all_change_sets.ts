// deno-lint-ignore-file no-explicit-any
import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";
import {
    sleepBetween,
} from "../test_helpers.ts";

export default async function abandon_all_change_sets(sdfApiClient: SdfApiClient) {
    await sleepBetween(0, 750);
    return abandon_all_change_sets_inner(sdfApiClient);
}

async function abandon_all_change_sets_inner(sdf: SdfApiClient) {

    const workspaceId = sdf.workspaceId;
    const data = await sdf.call({
        route: "list_open_change_sets",
        routeVars: {
            workspaceId,
        },
    });
    // loop through them and abandon each one that's not head
    assert(data.defaultChangeSetId, "Expected headChangeSetId");
    const changeSetsToAbandon = data.changeSets.filter((c) => c.id !== data.defaultChangeSetId);

    for (const changeSet of changeSetsToAbandon) {
        const changeSetId = changeSet.id;
       try{ await sdf.call({
            route: "abandon_change_set",
            body: {
                changeSetId,
            },
        });
       } catch(err) {
        console.log(err);
       }
}

}