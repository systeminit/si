// deno-lint-ignore-file no-explicit-any
import { SdfApiClient } from "../sdf_api_client.ts";
import {
    runWithTemporaryChangeset,
    sleepBetween,
    createComponent,
    getQualificationSummary,
    installModule,
    extractSchemaVariant,
    getSchemaVariants,
    getActions,
    getFuncs,
} from "../test_helpers.ts";

export default async function one_upper(sdfApiClient: SdfApiClient) {
    await sleepBetween(0, 750);
    return runWithTemporaryChangeset(sdfApiClient, one_upper_inner);
}

function calculate_offset(initial_x: number, offset_x: number) {
    return initial_x + offset_x
}
/// This test creates N chains of pairs, such that setting the attribute value on the first component
/// in the chain should propagate through the entire chain, adding 1 at each step.
async function one_upper_inner(sdf: SdfApiClient,
    changeSetId: string,
) {
    // install requisite modules
    const one_upper_module_with_quals = '01JKW4VKKJ0DCVHRH0HQ4D6ZG4'; // one upper with leaf funcs and validations
    const value_passer_module_with_validation = '01JKW4VX88MJ2R1EAV73H1NRQM'; // value passwer with leaf funcs and validations

    // decide whether you want qualifications/validations or not 
    // then install the modules by the module id. 
    await installModule(sdf, changeSetId, value_passer_module_with_validation);
    await installModule(sdf, changeSetId, one_upper_module_with_quals);

    // Edit these numbers for different amount of stress on the system
    const num_chains = 3;
    const num_component_pairs = 20; // note: 50 pairs actually results in 99 components as the chain ends with the value passer

    const { schemaVariants, newCreateComponentApi } = await getSchemaVariants(
        sdf,
        changeSetId,
    );


    const valueVariant = await extractSchemaVariant(
        schemaVariants,
        "ValuePasser",
        "KEEB",
    );
    const valueVariantId = valueVariant.schemaVariantId;
    const plusOneVariant = await extractSchemaVariant(
        schemaVariants,
        "OneUpper",
        "KEEB",
    );
    const plusOneVariantId = plusOneVariant.schemaVariantId;
    for (let j = 0; j < num_chains; j++) {
        let y: number;
        // each chain is in it's own 'row'
        y = j * 550;

        // create the first component and get the attribute value Id we'll be setting after the chain is created
        const first_value = await createComponent(sdf, changeSetId, valueVariantId, 0, y, undefined, newCreateComponentApi);
        const sourceInputProp = valueVariant.props.find((p) =>
            p.path === "/root/domain/input"
        );

        const sourcePropValues = await sdf.call({
            route: "get_property_values",
            routeVars: {
                componentId: first_value,
                changeSetId,
            },
        });
        let sourceAttributeValue;
        for (const attributeValue in sourcePropValues?.values) {
            if (
                sourcePropValues?.values[attributeValue]?.propId === sourceInputProp.id
            ) {
                sourceAttributeValue = attributeValue;
            }
        }
        let sourceAttributeValueParent;
        for (const attributeValue in sourcePropValues?.childValues) {
            const avChildren = sourcePropValues?.childValues[attributeValue] ?? [];
            if (avChildren.includes(sourceAttributeValue)) {
                sourceAttributeValueParent = attributeValue;
            }
        }
        // track the last component Id to create chains
        let last_value_id: string | undefined;
        last_value_id = first_value;
        // track x position. Y stays the same for a chain but we must increment X for every pair so they're not stacked on 
        // top of each other
        let x: number;
        x = 0;
        for (let i = 1; i < num_component_pairs; i++) {

            // create one-upper, which is a child of the first value passer
            const one_upper = await createComponent(sdf, changeSetId, plusOneVariantId, x, y, last_value_id, newCreateComponentApi);
            // offset next value passer we're creating
            const new_coords = calculate_offset(x, 600);
            const new_x = new_coords;

            // create next value passer, no parent
            const next_value = await createComponent(sdf, changeSetId, valueVariantId, new_x, y, undefined, newCreateComponentApi);

            // connect inner one-upper to the next value passer via sockets 
            const inputSocket = valueVariant.inputSockets.find((s) =>
                s.name === "input"
            );
            const outputSocket = plusOneVariant.outputSockets.find((s) =>
                s.name === "output"
            );
            const createConnectionPayload = {
                "fromComponentId": one_upper,
                "fromSocketId": outputSocket?.id,
                "toComponentId": next_value,
                "toSocketId": inputSocket?.id,
                "visibility_change_set_pk": changeSetId,
                "workspaceId": sdf.workspaceId,
            };
            await sdf.call({
                route: "create_connection",
                body: createConnectionPayload,
            });
            // put some more load on the system to see how response times are impacted by this activity
            await Promise.all([
                getActions(sdf, changeSetId),
                getFuncs(sdf, changeSetId),
                getQualificationSummary(sdf, changeSetId),
                // note: these two below do the same thing under the covers when you don't pass in a view id
                // choose your adventure!
                // sdf.call({
                //     route: "get_diagram",
                //     routeVars: {
                //         workspaceId: sdf.workspaceId,
                //         changeSetId,
                //     },
                // }),
                sdf.call({
                    route: "get_all_components_and_edges",
                    routeVars: {
                        workspaceId: sdf.workspaceId,
                        changeSetId,
                    },
                }),
            ]);

            // update values used on next loop!
            last_value_id = next_value;
            x = new_x;
        }
        // after chain is created, update attribute value for the first value passer
        const regionValue = "1";
        const updateValuePayload = {
            "attributeValueId": sourceAttributeValue,
            "parentAttributeValueId": sourceAttributeValueParent,
            "propId": sourceInputProp.id,
            "componentId": first_value,
            "value": regionValue,
            "isForSecret": false,
            "visibility_change_set_pk": changeSetId,
        };
        await sdf.call({
            route: "update_property_value",
            body: updateValuePayload,
        });

    }
    // after all chains are created and initial values set, wait for DVU. 
    // this times out sometimes, but we're still able to get what we need 
    // from telemetry
    await sdf.waitForDVURoots(changeSetId, 2000, 300000);
}