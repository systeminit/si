async function main({
    thisComponent
}: Input): Promise<Output> {
    // Get the name from the resourceId.
    const component = thisComponent;
    const name = component.properties?.si?.resourceId;
    if (!name) {
        return {
            status: "error",
            message: "No resourceId present",
        };
    }

    // Run the AWS CLI command.
    const cliInput = { name };
    const Region = component.properties?.domain?.extra?.Region ?? "";
    const child = await siExec.waitUntilEnd("aws", [
        "eks",
        "describe-cluster",
        "--cli-input-json",
        JSON.stringify(cliInput),
        "--region",
        Region,
    ], { stderr: ["inherit", "pipe"] });

    // Return an error if the CLI command failed. (Handle specific error cases here.)
    if (child.failed) {
        // Remove the payload if the resource no longer exists in AWS
        const NOT_FOUND_MESSAGE = "ResourceNotFoundException"
        if (child.stderr?.includes(NOT_FOUND_MESSAGE)) {
            console.log(`Resource not found upstream (${NOT_FOUND_MESSAGE}) so removing the resource.`)
            return {
                status: "ok",
                payload: null
            };
        }
        return {
            status: "error",
            message: child.message
        }
    }

    // Construct the SI properties by looking at the AWS resource
    const cluster = JSON.parse(child.stdout).cluster;
    let domain = {
        name: cluster.name,
        version: cluster.version ?? "",
        roleArn: cluster.roleArn ?? "",
        resourcesVpcConfig: {
            subnetIds: cluster.resourcesVpcConfig.subnetIds || [],
            securityGroupIds: cluster.resourcesVpcConfig.securityGroupIds || [],
            endpointPublicAccess: cluster.resourcesVpcConfig.endpointPublicAccess || false,
            endpointPrivateAccess: cluster.resourcesVpcConfig.endpointPrivateAccess || false,
            publicAccessCidrs: cluster.resourcesVpcConfig.publicAccessCidrs || []
        },
        kubernetesNetworkConfig: {
            serviceIpv4Cidr: cluster.kubernetesNetworkConfig?.serviceIpv4Cidr || "",
            ipFamily: cluster.kubernetesNetworkConfig?.ipFamily || ""
        },
        enabledLoggingTypes: (cluster.logging?.clusterLogging || [])
            .filter((log: any) => log.enabled)
            .flatMap((log: any) => log.types),
        tags: cluster.tags || {},
    };

    // Optional mapping for encryptionConfig, certificateAuthority, or other fields can be added similarly if required

    // Update component properties with the new domain, and queue a refresh.
    return {
        status: "ok",
        message: JSON.stringify(cluster),
        ops: {
            update: {
                self: {
                    properties: {
                        ...component?.properties,
                        domain: {
                            ...component?.properties?.domain,
                            ...domain,
                        },
                    }
                }
            },
            actions: {
                self: {
                    remove: ["create"],
                    add: ["refresh"],
                }
            }
        }
    };
}
