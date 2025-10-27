async function main(component: Input): Promise<Output> {
  const cluster = component.properties?.domain?.Cluster;
  const serviceName = component.properties?.domain?.ServiceName;
  const region = component.properties?.domain?.extra?.Region || "us-east-1";

  if (!cluster || !serviceName) {
    return {
      status: "error",
      message:
        "Missing required service configuration: cluster or serviceName not found",
    };
  }

  console.log(
    `Force deploying ECS service: ${serviceName} in cluster: ${cluster}`,
  );

  try {
    // Force a new deployment using AWS CLI
    const child = await siExec.waitUntilEnd("aws", [
      "ecs",
      "update-service",
      "--cluster",
      cluster,
      "--service",
      serviceName,
      "--force-new-deployment",
      "--region",
      region,
    ]);

    if (child.exitCode !== 0) {
      console.error("AWS CLI error:", child.stderr);
      return {
        status: "error",
        message:
          `Force deployment failed: AWS CLI exited with code ${child.exitCode}. Error: ${child.stderr}`,
      };
    }

    const response = JSON.parse(child.stdout);
    console.log("Force deployment initiated successfully");

    return {
      status: "ok",
      message:
        `Force deployment initiated for service ${serviceName}. Deployment ARN: ${
          response.service?.deployments?.[0]?.deploymentArn || "unknown"
        }`,
    };
  } catch (error) {
    console.error("Error during force deployment:", error);
    return {
      status: "error",
      message: `Force deployment failed: ${
        error instanceof Error ? error.message : "Unknown error"
      }`,
    };
  }
}
