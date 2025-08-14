// deno-lint-ignore-file no-explicit-any
import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";
import {
    runWithTemporaryChangeset,
    sleepBetween,
    createAsset,
    updateAssetCode,
    nilId,
    createQualification,
    eventualMVAssert,
    getViews,
} from "../test_helpers.ts";

export default async function emulate_authoring(sdfApiClient: SdfApiClient) {
    await sleepBetween(0, 750);
    return runWithTemporaryChangeset(sdfApiClient, emulate_authoring_inner);
}

async function emulate_authoring_inner(sdf: SdfApiClient,
    changeSetId: string,
) {

    // Get the views and find the default one
    const views = await getViews(sdf, changeSetId);
    const defaultView = views.find((v: any) => v.isDefault);
    assert(defaultView, "Expected to find a default view");

    // Create new asset
    let schemaVariantId = await createAsset(sdf, changeSetId, "doubler");

    // wait for and verify schema variant MV
    await eventualMVAssert(
        sdf,
        changeSetId,
        "SchemaVariant",
        schemaVariantId,
        (mv) => mv.id === schemaVariantId,
        "SchemaVariant MV should exist and have matching id"
    );

    // Update Code and Regenerate
    schemaVariantId = await updateAssetCode(sdf, changeSetId, schemaVariantId, doublerAssetCode);

    await eventualMVAssert(
        sdf,
        changeSetId,
        "SchemaVariant",
        schemaVariantId,
        (mv) => {
            const props = Object.values(mv.propTree?.props) || [];
            return mv.id === schemaVariantId &&
                props.some((p: any) => p.path === "root/domain/input") &&
                props.some((p: any) => p.path === "root/domain/doubled");
        },
        "SchemaVariant MV should exist and have added input/doubled props"
    );
    let doublerVariant = await sdf.mjolnir(changeSetId, "SchemaVariant", schemaVariantId);
    assert(doublerVariant?.propTree?.props, "Expected props list");

    let doublerProps = Object.values(doublerVariant.propTree.props);

    assert(Array.isArray(doublerProps), "Expected props to be an array");
    // Add an attribute function
    const outputProp = doublerProps.find((p: any) => p.path === "root/domain/doubled");
    assert(outputProp, "Expected to find doubled prop");
    let inputProp: any = doublerProps.find((p: any) => p.path === "root/domain/input");
    assert(inputProp, "Expected to find input prop");
    const args = [
        {
            name: "input",
            kind: "string",
            propId: inputProp.id,
            inputSocketId: null,
        },
    ];

    const doubleFuncId = await createAttributeFunction(sdf, changeSetId, "double", doublerVariant.id, doubleFuncCode, outputProp.id, args);
    let createComponentBody = {
        "schemaVariantId": schemaVariantId,
        "x": "0",
        "y": "0",
        "height": "0",
        "width": "0",
        "parentId": null,
        "schemaType": "installed"

    };

    // create a component for the doubler
    const createComponentResp = await sdf.call({
        route: "create_component_v2",
        routeVars: {
            workspaceId: sdf.workspaceId,
            changeSetId,
            viewId: defaultView.id,
        },
        body: createComponentBody,
    });
    const doublerComponentId = createComponentResp?.componentId;
    assert(doublerComponentId, "Expected to get a component id after creation");
    await eventualMVAssert(
        sdf,
        changeSetId,
        "Component",
        doublerComponentId,
        (mv) => mv.id === doublerComponentId && mv.qualificationTotals.succeeded === 0,
        "Component MV should exist"
    );

    // update input prop to be a number
    const updateRegionResponse = await sdf.call({
        route: "attributes",
        routeVars: {
            workspaceId: sdf.workspaceId,
            changeSetId,
            componentId: doublerComponentId,
        },
        body: {
            "/domain/input": "2",
        },
    });

    await eventualMVAssert(
        sdf,
        changeSetId,
        "AttributeTree",
        doublerComponentId,
        (mv) => Object.values(mv.attributeValues).some(
            (av: any) => av.path === "/domain/input" &&
                av.value === "2",
        ),
        "Expected value to be set for domain/input",
    );


    // now add a qualification and check that the component gets it
    const qualification = await createQualification(
        sdf,
        changeSetId, "doublerQualification",
        schemaVariantId,
        doublerQualificationCode
    );


    // it is failing
    await eventualMVAssert(
        sdf,
        changeSetId,
        "Component",
        doublerComponentId,
        (mv) => mv.qualificationTotals.failed === 1,
        "Component should have a failed qualification now",
    );


    await eventualMVAssert(
        sdf,
        changeSetId,
        "AttributeTree",
        doublerComponentId,
        (mv) => Object.values(mv.attributeValues).some(
            (av: any) => av.path === "/domain/doubled" &&
                av.value === "4",
        ),
        "Expected doubled attribute value to be 4",
    );

    // Now let's add a prop, regenerate, add an attribute func, and make sure it all works
    schemaVariantId = await updateAssetCode(sdf, changeSetId, schemaVariantId, doublerAssetCodeAddedTripled);
    await eventualMVAssert(
        sdf,
        changeSetId,
        "SchemaVariant",
        schemaVariantId,
        (mv) => Object.values(mv.propTree.props).some((p: any) => p.path === "root/domain/tripled"),
        "SchemaVariant should have new tripled prop"
    );

    doublerVariant = await sdf.mjolnir(changeSetId, "SchemaVariant", schemaVariantId);
    doublerProps = Object.values(doublerVariant.propTree.props);
    assert(Array.isArray(doublerProps), "Expected props to be an array");

    // Add an attribute function
    const tripleProp = doublerProps.find((p: any) => p.path === "root/domain/tripled");
    assert(tripleProp, "Expected to find output prop");
    inputProp = doublerProps.find((p: any) => p.path === "root/domain/input");
    assert(inputProp, "Expected to find input prop");
    const triplerArgs = [
        {
            name: "input",
            kind: "string",
            propId: inputProp.id,
            inputSocketId: null,
        },
    ];

    const tripleFuncId = await createAttributeFunction(sdf, changeSetId, "triple", schemaVariantId, tripleFuncCode, tripleProp.id, triplerArgs);

    // now ensure the component has the new prop too and it's value has been updated
    await eventualMVAssert(
        sdf,
        changeSetId,
        "AttributeTree",
        doublerComponentId,
        (mv) => Object.values(mv.attributeValues).some(
            (av: any) => av.path === "/domain/tripled" &&
                av.value === "6",
        ),
        "Expected tripled attribute value to be 6",
    );

}


const doubleFuncCode = `async function main(input: Input): Promise < Output > {
    const number = parseInt(input?.input);
    if (!number) {
        return String(0);      
    }
                
    return String(number * 2);
}
`;

const tripleFuncCode = `async function main(input: Input): Promise < Output > {
    const number = parseInt(input?.input);
    if (!number) {
        return String(0);      
    }
                
    return String(number * 3);
}
`;

const doublerAssetCode = `function main() {
    const asset = new AssetBuilder();

    const inputProp = new PropBuilder()
        .setName("input")
        .setKind("string")
        .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
        .build();
    asset.addProp(inputProp);

    const doublerProp = new PropBuilder()
        .setName("doubled")
        .setKind("string")
        .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
        .build();

    asset.addProp(doublerProp);

    return asset.build();
}`;

const doublerAssetCodeAddedTripled = `function main() {
    const asset = new AssetBuilder();

    const inputProp = new PropBuilder()
        .setName("input")
        .setKind("string")
        .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
        .build();
    asset.addProp(inputProp);

    const doublerProp = new PropBuilder()
        .setName("doubled")
        .setKind("string")
        .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
        .build();

    asset.addProp(doublerProp);

    const triplerProp = new PropBuilder()
        .setName("tripled")
        .setKind("string")
        .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
        .build();

    asset.addProp(triplerProp);

    return asset.build();
}`;

const doublerQualificationCode = `async function main(component: Input): Promise < Output > {

    const doubler = component?.domain?.doubler;
    if (doubler) {
        const val = parseInt(doubler);
        if (val > 0) {
            return {
                result: 'success',
                message: 'Component qualified'
            };
        }
    }

    return {
        result: 'failure',
        message: 'Component not qualified'
    };
}`;

// REQUEST HELPERS WITH VALIDATIONS

async function createAttributeFunction(sdf: SdfApiClient, changeSetId: string, name: string, schemaVariantId: string, code: string, outputLocationId: string, funcArguments: any[]) {
    const createFuncPayload = {
        name,
        displayName: name,
        description: "",
        binding: {
            funcId: nilId,
            schemaVariantId: schemaVariantId,
            bindingKind: "attribute",
            argumentBindings: [],
            propId: outputLocationId,
        },
        kind: "Attribute",
        requestUlid: changeSetId,
    };

    const createFuncResp = await sdf.call({
        route: "create_func",
        routeVars: {
            workspaceId: sdf.workspaceId,
            changeSetId,
        },
        body: createFuncPayload,
    });

    // now list funcs and let's make sure we see it
    const funcs = await sdf.call({
        route: "func_list",
        routeVars: {
            workspaceId: sdf.workspaceId,
            changeSetId,
        },
    });

    const createdFunc = funcs.find((f) => f.name === name);
    assert(createdFunc, "Expected to find newly created func");
    const funcId = createdFunc.funcId;
    const codePayload = {
        code,
        requestUlid: changeSetId,
    };

    // save the code
    const updateCodeResp = await sdf.call({
        route: "update_func_code",
        routeVars: {
            workspaceId: sdf.workspaceId,
            changeSetId,
            funcId,
        },
        body: codePayload,
    });

    // create func arguments
    let numArgs = 0;
    for (const funcArg of funcArguments) {
        // create the argument
        let argPayload = {
            created_at: new Date(),
            updated_at: new Date(),
            name: funcArg.name,
            id: nilId,
            kind: funcArg.kind,
            requestUlid: changeSetId,
        };
        const createArgResp = await sdf.call({
            route: "create_func_arg",
            routeVars: {
                workspaceId: sdf.workspaceId,
                changeSetId,
                funcId,
            },
            body: argPayload,
        });
        const args = createArgResp.arguments;
        assert(
            Array.isArray(createArgResp?.arguments),
            "Expected arguments list",
        );
        numArgs++;
        assert(createArgResp?.arguments.length === numArgs, `Expected ${numArgs} but found ${createArgResp?.arguments.length}`);
        const createdArg = args.find((arg) => arg.name === funcArg.name);
        const attributePrototypeArgumentId = createdArg.id;
        const attributePrototypeId = createArgResp?.bindings[0].attributePrototypeId;
        // now update the argument bindings

        const bindingPayload = {
            funcId,
            bindings: [
                {
                    bindingKind: "attribute",
                    attributePrototypeId: attributePrototypeId,
                    funcId,
                    propId: outputLocationId,
                    componentId: null,
                    outputSocketId: null,
                    schemaVariantId,
                    argumentBindings: [
                        {
                            funcArgumentId: createdArg.id,
                            propId: funcArg.propId,
                            attributePrototypeArgumentId: attributePrototypeArgumentId,
                            inputSocketId: funcArg.inputSocketId,
                        }
                    ]
                }
            ]
        }
        const updateBindingResp = await sdf.call({
            route: "create_func_binding",
            routeVars: {
                workspaceId: sdf.workspaceId,
                changeSetId,
                funcId,
            },
            body: bindingPayload,
        });

        assert(
            Array.isArray(updateBindingResp),
            "Expected bindings list",
        );
        assert(
            Array.isArray(updateBindingResp[0].argumentBindings),
            "Expected argument bindings list",
        );
        assert(
            updateBindingResp[0].argumentBindings.length === numArgs,
            "Expected argument bindings list",
        );
    }

    return funcId;

}


