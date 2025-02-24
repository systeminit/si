// deno-lint-ignore-file no-explicit-any
import { SdfApiClient } from "../sdf_api_client.ts";
import {
    runWithTemporaryChangeset,
    sleepBetween,
    createComponent,
    installModule,
    getSchemaVariants,
} from "../test_helpers.ts";

export default async function install_module(sdfApiClient: SdfApiClient) {
    await sleepBetween(0, 750);
    return runWithTemporaryChangeset(sdfApiClient, install_module_inner);
}

async function install_module_inner(sdf: SdfApiClient,
    changeSetId: string,
) {
    // decide how many modules you want to install
    const num_to_install = 500;

    // get the list of builtins from the module index - note we're doing this and not hitting the schema variants route 
    // so we can separately track install from component creation

    const resp = await fetch("https://module-index.systeminit.com/builtins", {
        "headers": {
            "accept": "application/json, text/plain, */*",
            "accept-language": "en-US,en;q=0.9",
            "Cache-Control": "no-cache",
            "User-Agent": "si.git/api-tests (support@systeminit.com)",
        },
        "body": null,
        "method": "GET"
    });
    const builtins = await resp.json();


    // first install n builtin modules
    let installed = 0;
    while (installed < num_to_install) {
        let module_to_install = builtins.modules[installed];
        console.log(`INSTALLING MODULE ${installed} of ${num_to_install}`);
        await installModule(sdf, changeSetId, module_to_install.id);
        installed++;
    }
    console.log(`FINISHED INSTALLING ${num_to_install} MODULES`);
    // Now create one component for each installed module

    const { schemaVariants, newCreateComponentApi } = await getSchemaVariants(
        sdf,
        changeSetId,
    );
    let num_created = 0;
    for (const variant of schemaVariants) {
        // create space between each one as a factor of the number being created
        console.log(`CREATING COMPONENT ${num_created} of ${num_to_install}`);
        await createComponent(sdf, changeSetId, variant.schemaVariantId, num_created * 450, 0, undefined, newCreateComponentApi);
        num_created++;
    }
    console.log(`FINISHED CREATING ${num_created} COMPONENTS`);

    // wait for dvu!
    await sdf.waitForDVURoots(changeSetId, 2000, 300000);
}