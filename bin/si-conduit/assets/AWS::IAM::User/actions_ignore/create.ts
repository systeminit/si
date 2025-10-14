async function main(component: Input): Promise<Output> {
  const identityType = component.properties?.domain?.IdentityType;
  if (!identityType) {
    return {
      status: "error",
      message: "schema using this function must have an IdentityType property.",
      payload: component.properties,
    };
  }
  if (component.properties.resource?.payload) {
    return {
      status: "error",
      message: "Resource already exists",
      payload: component.properties.resource.payload,
    };
  }

  let identityField = "";
  let codeGen = "";
  let responseField = "";
  if (identityType === "user") {
    identityField = "UserName";
    codeGen = "awsIamUserCodeGen";
    responseField = "User";
  } else if (identityType === "group") {
    identityField = "GroupName";
  } else if (identityType === "role") {
    identityField = "RoleName";
    codeGen = "awsIamRoleCodeGen";
    responseField = "Role";
  }
  const identity = _.get(component, ["properties", "domain", identityField]) ||
    "";

  const command = `create-${identityType}`;

  let code = component.properties.code?.[codeGen]?.code as string;
  if (identityType === "role") {
    const codeObj = JSON.parse(code);
    _.set(
      codeObj,
      ["AssumeRolePolicyDocument"],
      JSON.stringify(codeObj.AssumeRolePolicyDocument),
    );
    code = JSON.stringify(codeObj, null, 2);
  }
  const args = [
    "iam",
    command,
    "--cli-input-json",
    code,
  ];
  const child = await siExec.waitUntilEnd("aws", args);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      status: "error",
      message:
        `Unable to create ${identityType} ${identity}; AWS CLI 2 exited with non zero code: ${child.exitCode}`,
    };
  }

  const response = JSON.parse(child.stdout);
  const payload = _.get(response, [responseField]);

  return {
    payload,
    status: "ok",
  };
}
