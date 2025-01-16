async function main({
    thisComponent
}: Input): Promise < Output > {
    const component = thisComponent.properties;
    const region = _.get(component, ["domain", "extra", "Region"], "");
    const awsResourceType = _.get(component, ["domain", "extra", "AwsResourceType"], "");

    let resourceList = [];
    let finished = false;
    let nextToken = "";
    const create = {};
    const actions = {};

    while (!finished) {
        const listArgs = [
            "cloudcontrol",
            "list-resources",
            "--region",
            region,
            "--type-name",
            awsResourceType,
        ];
        if (nextToken) {
            listArgs.push(nextToken);
        }

        const listChild = await siExec.waitUntilEnd("aws", listArgs);

        if (listChild.exitCode !== 0) {
            console.log("Failed to list cloud control resources");
            console.log(listChild.stdout);
            console.error(listChild.stderr);
            return {
                status: "error",
                message: `Resource list error; exit code ${listChild.exitCode}.\n\nSTDOUT:\n\n${listChild.stdout}\n\nSTDERR:\n\n${listChild.stderr}`,
            };
        }
        const listResponse = JSON.parse(listChild.stdout);
        console.log(listResponse);
        if (listResponse["NextToken"]) {
            nextToken = listResponse["NextToken"];
        } else {
            finished = true;
        }
        resourceList = _.union(resourceList, listResponse["ResourceDescriptions"]);
    }

    let x = 100;
    for (const resource of resourceList) {

        let resourceId = resource["Identifier"];
        console.log(`Importing ${resourceId}`)

        const child = await siExec.waitUntilEnd("aws", [
            "cloudcontrol",
            "get-resource",
            "--region",
            region,
            "--type-name",
            awsResourceType,
            "--identifier",
            resourceId,
        ]);

        if (child.exitCode !== 0) {
            console.log("Failed to import cloud control resource");
            console.log(child.stdout);
            console.error(child.stderr);
            return {
                status: "error",
                message: `Import error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
            };
        }

        const resourceResponse = JSON.parse(child.stdout);
        const resourceProperties = JSON.parse(resourceResponse["ResourceDescription"]["Properties"])
        console.log(resourceProperties);

        let properties = {
          si: {
            resourceId,
          }
        };
        for (const rprop of Object.keys(resourceProperties)) {
            const category = _.get(component, ["domain", "extra", "AwsFieldMap", rprop]);
            if (category) {
                _.set(properties, ["domain", category, rprop], _.get(resourceProperties, rprop));
            }
        }
        console.log(properties);
        create[resourceId] = {
            properties,
            geometry: {
                x,
                y: 500.0
            }
        };
        actions[resourceId] = {
          add: ["refresh"],
          remove: ["create"],
        };
        x = x + 250.0;
    }


    return {
        status: "ok",
        message: `Discovered ${resourceList.length} Components`,
        ops: {
            create,
            actions,
        },
    }
}
