async function main(component: Input): Promise<Output> {
  const identityChild = await siExec.waitUntilEnd("aws", [
    "sts",
    "get-caller-identity",
    "--region",
    "us-east-1",
    "--output",
    "json",
  ]);

  if (identityChild.exitCode !== 0) {
    return {
      result: "failure",
      message: `Failed to get current identity: ${identityChild.stderr}`,
    };
  }

  const identity = JSON.parse(identityChild.stdout);
  const userArn = convertAssumedRoleArnToRoleArn(identity.Arn);

  // Let's only match on the root account
  // Not any permutation like `johnroot`
  if (userArn.endsWith(":root")) {
    return {
      result: "success",
      message: `${userArn} is a root account!`,
    };
  }

  const permsMap = JSON.parse(_.get(component, [
    "domain",
    "extra",
    "AwsPermissionsMap",
  ]));

  const allPermissions: string[] = [];
  Object.keys(permsMap).forEach((operation) => {
    if (permsMap[operation].permissions) {
      allPermissions.push(...permsMap[operation].permissions);
    }
  });

  const uniquePermissions = [...new Set(allPermissions)];

  const permissionChild = await siExec.waitUntilEnd("aws", [
    "iam",
    "simulate-principal-policy",
    "--policy-source-arn",
    userArn,
    "--action-names",
    ...uniquePermissions,
    "--output",
    "json",
  ]);

  if (permissionChild.exitCode !== 0) {
    return {
      result: "failure",
      message: `Failed to check permissions: ${permissionChild.stderr}`,
    };
  }

  const simulateResult = JSON.parse(permissionChild.stdout);
  const evaluationResults = simulateResult.EvaluationResults;

  const permissionMap = {};
  for (const result of evaluationResults) {
    permissionMap[result.EvalActionName] = result.EvalDecision === "allowed";
  }

  const results = {};
  let allGranted = true;

  Object.keys(permsMap).forEach((operation) => {
    const permissions = permsMap[operation].permissions || [];
    const missingPermissions = [];
    const allowed = {};

    permissions.forEach((permission) => {
      allowed[permission] = permissionMap[permission] || false;
      if (!allowed[permission]) {
        missingPermissions.push(permission);
      }
    });

    const operationAllGranted = missingPermissions.length === 0;
    if (!operationAllGranted) {
      allGranted = false;
    }

    results[operation] = {
      allowed,
      allGranted: operationAllGranted,
      missingPermissions,
    };

    console.log(
      `${operation}: ${
        operationAllGranted
          ? "[OK] All granted!"
          : "[WARN] Missing permissions:"
      }`,
    );
    if (missingPermissions.length > 0) {
      console.log(`${missingPermissions.join("\n")}`);
    }
  });

  const missingPermissionsSet = new Set(
    Object.values(results)
      .flatMap((result) => result.missingPermissions),
  );
  const missingPermissionsList = Array.from(missingPermissionsSet);

  return {
    result: allGranted ? "success" : "warning",
    message: allGranted
      ? `All ${uniquePermissions.length} permissions granted for user ${identity.UserId}`
      : `Missing permissions: ${
        missingPermissionsList.join(", ")
      }. Checked ${uniquePermissions.length} permissions for user ${identity.UserId}.`,
  };
}

function convertAssumedRoleArnToRoleArn(arnString: string): string {
  // Check if this is an assumed role ARN from SSO
  if (arnString.includes(":assumed-role/")) {
    const match = arnString.match(
      /arn:aws:sts::(\d+):assumed-role\/([^/]+)\/.*/,
    );
    if (match) {
      const accountId = match[1];
      const roleName = match[2];
      const roleArn = `arn:aws:iam::${accountId}:role/${roleName}`;
      console.log(`Converted assumed role ARN to IAM role ARN: ${roleArn}`);
      return roleArn;
    }
  }

  return arnString;
}
