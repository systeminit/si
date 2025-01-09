async function main(component: Input): Promise < Output > {
    if (!component.properties.resource?.payload) {
        return {
            status: "error",
            message: "Resource must exist to be updated",
            payload: component.properties.resource.payload,
        };
    }

    let resourceId = component.properties?.si?.resourceId;

    const refreshChild = await siExec.waitUntilEnd("aws", [
        "cloudcontrol",
        "get-resource",
        "--region",
        _.get(component, "properties.domain.extra.Region", ""),
        "--type-name",
        _.get(component, "properties.domain.extra.AwsResourceType", ""),
        "--identifier",
        resourceId,
    ]);

    if (refreshChild.exitCode !== 0) {
        console.log("Failed to refresh cloud control resource");
        console.log(refreshChild.stdout);
        console.error(refreshChild.stderr);
        // FIXME: should track down what happens when the resource doesnt exist
        //if (child.stderr.includes("ResourceNotFoundException")) {
        //    console.log("EKS Cluster not found  upstream (ResourceNotFoundException) so removing the resource")
        //    return {
        //        status: "ok",
        //        payload: null,
        //    };
        //}
        return {
            status: "error",
            message: `Update error while fetching current state; exit code ${refreshChild.exitCode}.\n\nSTDOUT:\n\n${refreshChild.stdout}\n\nSTDERR:\n\n${refreshChild.stderr}`,
        };
    }

    const resourceResponse = JSON.parse(refreshChild.stdout);
    const currentState = JSON.parse(resourceResponse["ResourceDescription"]["Properties"]);

    const desiredProps: Array < Record < string, any >> = _.get(component, ["properties", "domain", "Updateable"]);

    const desiredState = _.cloneDeep(currentState);
    _.merge(desiredState, desiredProps);
    const patch = jsonpatch.compare(currentState, desiredState, true);
    console.log("Computed patch", patch);

    const child = await siExec.waitUntilEnd("aws", [
        "cloudcontrol",
        "update-resource",
        "--region",
        _.get(component, "properties.domain.extra.Region", ""),
        "--type-name",
        _.get(component, "properties.domain.extra.AwsResourceType", ""),
        "--identifier",
        resourceId,
        "--patch-document",
        JSON.stringify(patch),
    ]);

    if (child.exitCode !== 0) {
        console.error(child.stderr);
        return {
            status: "error",
            message: `Unable to update; AWS CLI 2 exited with non zero code: ${child.exitCode}`,
        };
    }

    const progressEvent = JSON.parse(child.stdout);
    console.log("Progress Event", progressEvent);

    const delay = (time: number) => {
        return new Promise(res => {
            setTimeout(res, time)
        })
    };

    let finished = false;
    let success = false;
    let wait = 1000;
    const upperLimit = 10000;
    let message = "";
    let identifier = "";

    while (!finished) {
        const child = await siExec.waitUntilEnd("aws", [
            "cloudcontrol",
            "get-resource-request-status",
            "--region",
            _.get(component, "properties.domain.extra.Region", ""),
            "--request-token",
            _.get(progressEvent, ["ProgressEvent", "RequestToken"]),
        ]);

        if (child.exitCode !== 0) {
            console.error(child.stderr);
            return {
                status: "error",
                message: `Unable to create; AWS CLI 2 exited with non zero code: ${child.exitCode}`,
            };
        }
        const currentProgressEvent = JSON.parse(child.stdout);
        console.log("Current Progress", currentProgressEvent);
        const operationStatus = currentProgressEvent["ProgressEvent"]["OperationStatus"];
        if (operationStatus == "SUCCESS") {
            finished = true;
            success = true;
            identifier = currentProgressEvent["ProgressEvent"]["Identifier"];
        } else if (operationStatus == "FAILED") {
            finished = true;
            success = false;
            message = currentProgressEvent["ProgressEvent"]["StatusMessage"] || currentProgressEvent["ProgressEvent"]["ErrorCode"];
        } else if (operationStatus == "CANCEL_COMPLETE") {
            finished = true;
            success = false;
            message = "Operation Canceled by API or AWS.";
        }

        if (!finished) {
            console.log("\nWaiting to check status!\n");
            await delay(wait);
            if (wait != upperLimit) {
                wait = wait + 1000;
            }
        }
    }

    if (success) {
        const child = await siExec.waitUntilEnd("aws", [
            "cloudcontrol",
            "get-resource",
            "--region",
            _.get(component, "properties.domain.extra.Region", ""),
            "--type-name",
            _.get(component, "properties.domain.extra.AwsResourceType", ""),
            "--identifier",
            identifier,
        ]);

        if (child.exitCode !== 0) {
            console.log("Failed to refresh cloud control resource");
            console.log(child.stdout);
            console.error(child.stderr);
            // FIXME: should track down what happens when the resource doesnt exist
            //if (child.stderr.includes("ResourceNotFoundException")) {
            //    console.log("EKS Cluster not found  upstream (ResourceNotFoundException) so removing the resource")
            //    return {
            //        status: "ok",
            //        payload: null,
            //    };
            //}
            return {
                status: "error",
                payload: _.get(component, "properties.resource.payload"),
                message: `Refresh error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
            };
        }

        const resourceResponse = JSON.parse(child.stdout);
        const payload = JSON.parse(resourceResponse["ResourceDescription"]["Properties"])
        return {
            payload,
            status: "ok",
        };
    } else {
        return {
            message,
            payload: _.get(component, "properties.resource.payload"),
            status: "error",
        }
    }
}
