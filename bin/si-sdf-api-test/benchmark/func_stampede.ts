// deno-lint-ignore-file no-explicit-any
import { SdfApiClient } from "../sdf_api_client.ts";
import {
    runWithTemporaryChangeset,
    sleepBetween,
    createComponent,
    extractSchemaVariant,
    getSchemaVariants,
    createQualification,
    testExecuteFunc,
    getFuncRun,
    installModule,
} from "../test_helpers.ts";

export default async function func_stampede(sdfApiClient: SdfApiClient) {
    await sleepBetween(0, 750);
    return runWithTemporaryChangeset(sdfApiClient, func_stampede_inner);
}


/// This test creates N chains of pairs, such that setting the attribute value on the first component
/// in the chain should propagate through the entire chain, adding 1 at each step.
async function func_stampede_inner(sdf: SdfApiClient,
    changeSetId: string,
) {
    // install requisite modules if necessary
    const one_upper_module_with_quals = '01JKW4VKKJ0DCVHRH0HQ4D6ZG4'; // one upper with leaf funcs and validations
    await installModule(sdf, changeSetId, one_upper_module_with_quals);

    // Edit this number for different amount of stress on the system
    const num_funcs = 50;
    const { schemaVariants, newCreateComponentApi } = await getSchemaVariants(
        sdf,
        changeSetId,
    );

    const plusOneVariant = await extractSchemaVariant(
        schemaVariants,
        "OneUpper",
        "KEEB",
    );

    const qualificationCode = `function main(component: Input) {
    const fakeObject = {
        inputNumber: component.domain?.input,
        text: 'Lorem ipsum odor amet, consectetuer adipiscing elit.'
    };

    // Logging thousands of messages
    for (let i = 0; i < 10000; i++) {
        const val = i;
        console.log('Current fakeObject is {}', fakeObject);
    }

    return {
        result: 'success',
        message: 'Qualified',
    }; 
    }`;

    const plusOneVariantId = plusOneVariant.schemaVariantId;

    // unlock schema variant
    const plusOneVariantUnlocked = await sdf.call({
        route: "create_unlocked_copy", routeVars: {
            workspaceId: sdf.workspaceId,
            changeSetId,
            schemaVariantId: plusOneVariantId,
        },
    });
    const plusOneVariantIdUnlocked = plusOneVariantUnlocked.schemaVariantId;

    // create a new qualification func with lots of log lines
    const qualificationId = await createQualification(
        sdf,
        changeSetId, "loggyQualification",
        plusOneVariantIdUnlocked,
        qualificationCode
    );

    // create a component (needed to test execute the new qualification)
    const one_upper = await createComponent(sdf, changeSetId, plusOneVariantIdUnlocked, 0, 0, undefined, newCreateComponentApi);
    let funcRunsToCheck: string[] = [];

    // Create an array of promises for all test executions
    const testExecutePromises = Array.from({ length: num_funcs }, async (_, j) => {
        const args = {
            domain: {
                computed: "2",
                input: "1",
            }
        };
        const resp = await testExecuteFunc(sdf, changeSetId, qualificationId, one_upper, qualificationCode, args);
        console.log(`Dispatched Func Number ${j} with id ${resp.funcRunId}`);
        return resp.funcRunId;
    });

    // Wait for all promises to resolve and collect the funcRunIds
    funcRunsToCheck = await Promise.all(testExecutePromises);

    // wait for functions to complete!
    await waitForFuncRuns(sdf, changeSetId, funcRunsToCheck, 1000000);
}


export enum FuncRunState {
    Created = "Created",
    Dispatched = "Dispatched",
    Running = "Running",
    PostProcessing = "Postprocessing",
    Failure = "Failure",
    Success = "Success",
}
async function waitForFuncRuns(sdf: SdfApiClient, changeSetId: string, funcRunIds: string[], timeoutMs: number) {
    const start = Date.now();
    var numWaiting = funcRunIds.length;
    const pendingRuns = new Set(funcRunIds);
    while (pendingRuns.size > 0) {
        if (Date.now() - start > timeoutMs) {
            throw new Error('Timeout waiting for func runs to complete');
        }
        for (const funcRunId of [...pendingRuns]) {

            const result = await getFuncRun(sdf, changeSetId, funcRunId);
            console.log(`Func Run ${funcRunId} has state: ${result.funcRun?.state}`);
            if (result.funcRun?.state === FuncRunState.Success) {
                numWaiting--;
                console.log(`Func Run ${funcRunId} Succeeded. Waiting for ${numWaiting} more...`)
                pendingRuns.delete(funcRunId);
            }
        }
        await new Promise(resolve => setTimeout(resolve, 1000));
    }
    console.log("All funcs ran to completion! Woot!");
}