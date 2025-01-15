async function main(component: Input): Promise < Output > {
    if (component.properties.resource?.payload) {
        return {
            status: "error",
            message: "Resource already exists",
            payload: component.properties.resource.payload,
        };
    }

    const codeString = component.properties.code?.["awsCloudControlCreate"]?.code;
    console.log("code", codeString);
    const domain = component.properties?.domain;
    const code = JSON.parse(codeString);
    const inputObject = {
        TypeName: code["TypeName"],
        DesiredState: JSON.stringify(code["DesiredState"]),
    }
    const inputJson = JSON.stringify(inputObject);

    const child = await siExec.waitUntilEnd("aws", [
        "cloudcontrol",
        "create-resource",
        "--region",
        domain?.extra?.Region || "",
        "--cli-input-json",
        inputJson || "",
    ]);

    if (child.exitCode !== 0) {
        console.error(child.stderr);
        return {
            status: "error",
            message: `Unable to create; AWS CLI 2 exited with non zero code: ${child.exitCode}`,
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
            domain?.extra?.Region || "",
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
        return {
            resourceId: identifier,
            status: "ok",
        }
    } else {
        return {
            message,
            status: "error",
        }
    }
}
