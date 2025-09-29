async function main(component: Input): Promise<Output> {
  const props = component.domain || {};
  const preSharedKey = props.PreSharedKey;

  // If PreSharedKey is not provided, it's valid as it's optional
  if (
    preSharedKey === undefined || preSharedKey === null || preSharedKey === ""
  ) {
    return {
      result: "success",
      message: "Component qualified",
    };
  }

  // Check length constraints (8 to 64 characters)
  if (preSharedKey.length < 8 || preSharedKey.length > 64) {
    return {
      result: "failure",
      message: "PreSharedKey must be between 8 and 64 characters in length",
    };
  }

  // Check if it starts with zero
  if (preSharedKey.startsWith("0")) {
    return {
      result: "failure",
      message: "PreSharedKey cannot start with zero (0)",
    };
  }

  // Check allowed characters (alphanumeric, periods, and underscores)
  const allowedPattern = /^[a-zA-Z0-9._]+$/;
  if (!allowedPattern.test(preSharedKey)) {
    return {
      result: "failure",
      message:
        "PreSharedKey can only contain alphanumeric characters, periods (.), and underscores (_)",
    };
  }

  return {
    result: "success",
    message: "Component qualified",
  };
}
