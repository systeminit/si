async function main({
    thisComponent
}: Input): Promise<Output> {

    const component = thisComponent.properties;
    let clusterName = _.get(component, ["si", "resourceId"]);
    const region = component.domain?.extra?.Region || "";

    if (!clusterName) {
        return {
            status: "error",
            message: "Cluster Name is required for importing the resource.",
        };
    }

    // Fetch the EKS Cluster details using AWS CLI
    const eksClusterResp = await siExec.waitUntilEnd("aws", [
        "eks",
        "describe-cluster",
        "--region",
        region,
        "--name",
        clusterName,
    ]);

    if (eksClusterResp.exitCode !== 0) {
        console.error(eksClusterResp.stderr);
        return {
            status: "error",
            message: `Unable to fetch EKS cluster details: AWS CLI exited with non-zero code ${eksClusterResp.exitCode} ${eksClusterResp}`,
        };
    }

    const eksCluster = JSON.parse(eksClusterResp.stdout).cluster;

    // Map EKS cluster details to component properties
    component["domain"]["name"] = eksCluster.name || "";
    component["domain"]["version"] = eksCluster.version || "";
    component["domain"]["roleArn"] = eksCluster.roleArn || "";

    // Map resourcesVpcConfig properties
    component["domain"]["resourcesVpcConfig"] = {
        subnetIds: eksCluster.resourcesVpcConfig.subnetIds || [],
        securityGroupIds: eksCluster.resourcesVpcConfig.securityGroupIds || [],
        endpointPublicAccess: eksCluster.resourcesVpcConfig.endpointPublicAccess || false,
        endpointPrivateAccess: eksCluster.resourcesVpcConfig.endpointPrivateAccess || false,
        publicAccessCidrs: eksCluster.resourcesVpcConfig.publicAccessCidrs || []
    };

    // Map kubernetesNetworkConfig properties
    component["domain"]["kubernetesNetworkConfig"] = {
        serviceIpv4Cidr: eksCluster.kubernetesNetworkConfig?.serviceIpv4Cidr || "",
        ipFamily: eksCluster.kubernetesNetworkConfig?.ipFamily || ""
    };

    // Map enabled logging types
    component["domain"]["enabledLoggingTypes"] = eksCluster.logging?.clusterLogging
        .filter((log: any) => log.enabled)
        .flatMap((log: any) => log.types) || [];

    // Map tags
    component["domain"]["tags"] = eksCluster.tags || {};

    // Optional mapping for encryptionConfig, certificateAuthority, or other fields can be added similarly if required

    // Return the updated component
    return {
        status: "ok",
        message: JSON.stringify(eksCluster),
        ops: {
            update: {
                self: {
                    properties: {
                        ...component, // Push updated component properties back onto the tree
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
