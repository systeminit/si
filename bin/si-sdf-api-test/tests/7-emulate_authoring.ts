// deno-lint-ignore-file no-explicit-any
import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";
import {
    runWithTemporaryChangeset,
    sleepBetween,
    createComponent,
    createAsset,
    updateAssetCode,
    nilId,
    getQualificationSummary,
    getPropertyEditor,
    attributeValueIdForPropPath,
    setAttributeValue,
    createQualification,
} from "../test_helpers.ts";

export default async function emulate_authoring(sdfApiClient: SdfApiClient) {
    await sleepBetween(0, 750);
    return runWithTemporaryChangeset(sdfApiClient, emulate_authoring_inner);
}

async function emulate_authoring_inner(sdf: SdfApiClient,
    changeSetId: string,
) {
    sdf.listenForDVUs();

    // Create new asset
    let schemaVariantId = await createAsset(sdf, changeSetId, "doubler");

    // Update Code and Regenerate
    schemaVariantId = await updateAssetCode(sdf, changeSetId, schemaVariantId, doublerAssetCode);

    let doublerVariant = await sdf.call({
        route: "get_variant", routeVars: {
            workspaceId: sdf.workspaceId,
            changeSetId,
            schemaVariantId,
        },
    });
    assert(doublerVariant?.props, "Expected props list");
    assert(doublerVariant?.inputSockets, "Expected input sockets list");
    assert(doublerVariant?.inputSockets.length === 0, "Expected no input sockets");
    assert(doublerVariant?.outputSockets, "Expected output sockets list");
    assert(doublerVariant?.outputSockets.length === 0, "Expected no output sockets");

    // Add an attribute function
    const outputProp = doublerVariant.props.find((p) => p.path === "/root/domain/doubled");
    assert(outputProp, "Expected to find output prop");
    let inputProp = doublerVariant.props.find((p) => p.path === "/root/domain/input");
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


    // create a component for the doubler
    const doublerComponentId = await createComponent(sdf, changeSetId, doublerVariant.schemaVariantId, 0, 0, undefined, true);

    // update input prop to be a number
    const { values: doublerPropValues } = await getPropertyEditor(
        sdf,
        changeSetId,
        doublerComponentId,
    );


    const { attributeValueId, parentAttributeValueId, propId } =
        attributeValueIdForPropPath(
            "/root/domain/input",
            doublerVariant.props,
            doublerPropValues,
        );
    const inputValue = "2";
    await setAttributeValue(
        sdf,
        changeSetId,
        doublerComponentId,
        attributeValueId,
        parentAttributeValueId,
        propId,
        inputValue,
    );


    // now add a qualification and check that the component gets it
    const qualification = await createQualification(
        sdf,
        changeSetId, "doublerQualification",
        schemaVariantId,
        doublerQualificationCode
    );

    await sdf.waitForDVURoots(changeSetId, 2000, 60000);

    // now let's check everything
    const qualSummary = await getQualificationSummary(sdf, changeSetId);
    assert(
        Array.isArray(qualSummary?.components),
        "Expected arguments list",
    );
    const componentQual = qualSummary?.components.find((c) => c.componentId === doublerComponentId);
    assert(componentQual, "Expected to find qualification summary for created component");
    assert(componentQual?.failed === 1, "Expected to have one qualification failed as it doesn't yet have the value set");
    const { values: doublerValues } = await getPropertyEditor(
        sdf,
        changeSetId,
        doublerComponentId,
    );

    const doubleAV =
        attributeValueIdForPropPath(
            "/root/domain/doubled",
            doublerVariant.props,
            doublerValues,
        );

    assert(doubleAV?.value === "4", `Expected doubled attribute value to be 4 but found ${doubleAV?.value}`);

    // Now let's add a prop, regenerate, add an attribute func, and make sure it all works
    schemaVariantId = await updateAssetCode(sdf, changeSetId, schemaVariantId, doublerAssetCodeAddedTripled);
    doublerVariant = await sdf.call({
        route: "get_variant", routeVars: {
            workspaceId: sdf.workspaceId,
            changeSetId,
            schemaVariantId,
        },
    });


    // Add an attribute function
    const tripleProp = doublerVariant.props.find((p) => p.path === "/root/domain/tripled");
    assert(tripleProp, "Expected to find output prop");
    inputProp = doublerVariant.props.find((p) => p.path === "/root/domain/input");
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

    const newDoublerVariant = await sdf.call({
        route: "get_variant", routeVars: {
            workspaceId: sdf.workspaceId,
            changeSetId,
            schemaVariantId,
        },
    });
    await sdf.waitForDVURoots(changeSetId, 2000, 60000);

    // Let's make sure this updated accurately on the component
    const { values: triplerValues } = await getPropertyEditor(
        sdf,
        changeSetId,
        doublerComponentId,
    );

    const tripleAV =
        attributeValueIdForPropPath(
            "/root/domain/tripled",
            newDoublerVariant.props,
            triplerValues,
        );
    assert(tripleAV?.value === "6", `Expected doubled attribute value to be 6  but found ${tripleAV?.value}`);

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


