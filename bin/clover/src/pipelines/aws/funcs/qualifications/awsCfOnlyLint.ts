async function main(component: Input): Promise<Output> {
  const resourceBody = component.domain?.CloudFormationResourceBody;

  if (!resourceBody) {
    return {
      result: "warning",
      message: "No CloudFormationResourceBody found - nothing to validate",
    };
  }

  // Parse the resource body
  let resource;
  try {
    resource = JSON.parse(resourceBody);
  } catch (e) {
    return {
      result: "failure",
      message: `Invalid JSON in CloudFormationResourceBody: ${e.message}`,
    };
  }

  // Validate basic structure
  if (!resource.Type) {
    return {
      result: "failure",
      message: "CloudFormation resource is missing 'Type' field",
    };
  }

  if (!resource.Properties) {
    return {
      result: "warning",
      message: "CloudFormation resource has no 'Properties' - this may be intentional",
    };
  }

  // Build a full CloudFormation template for validation
  const cloudFormationTemplate = {
    AWSTemplateFormatVersion: "2010-09-09",
    Resources: {
      CfnResource: resource,
    },
  };

  const templateJson = JSON.stringify(cloudFormationTemplate, null, 2);

  // Try to run cfn-lint if available
  const lintResult = await siExec.waitUntilEnd("cfn-lint", [
    "--template",
    "/dev/stdin",
    "--format",
    "json",
  ], {
    input: templateJson,
  });

  if (lintResult.exitCode === 127) {
    // cfn-lint not installed - do basic validation only
    console.log("cfn-lint not available, performing basic validation only");
    return {
      result: "success",
      message: `CloudFormation resource structure is valid (Type: ${resource.Type}). Install cfn-lint for deeper validation.`,
    };
  }

  if (lintResult.exitCode === 0 && (!lintResult.stdout || lintResult.stdout.trim() === "[]")) {
    return {
      result: "success",
      message: `CloudFormation template passed cfn-lint validation (Type: ${resource.Type})`,
    };
  }

  // Parse lint results
  try {
    const lintErrors = JSON.parse(lintResult.stdout || "[]");

    if (lintErrors.length === 0) {
      return {
        result: "success",
        message: `CloudFormation template passed cfn-lint validation (Type: ${resource.Type})`,
      };
    }

    // Categorize errors vs warnings
    const errors = lintErrors.filter((e: any) => e.Level === "Error");
    const warnings = lintErrors.filter((e: any) => e.Level === "Warning");

    const messages = lintErrors.map((e: any) =>
      `[${e.Level}] ${e.Rule.Id}: ${e.Message}`
    );

    console.log("cfn-lint results:");
    messages.forEach((m: string) => console.log(m));

    if (errors.length > 0) {
      return {
        result: "failure",
        message: `CloudFormation validation failed: ${errors.length} error(s), ${warnings.length} warning(s). First error: ${errors[0].Message}`,
      };
    }

    return {
      result: "warning",
      message: `CloudFormation validation passed with ${warnings.length} warning(s): ${warnings[0]?.Message || "See logs for details"}`,
    };
  } catch (e) {
    // If we can't parse the output, check if there was stderr
    if (lintResult.stderr) {
      return {
        result: "failure",
        message: `cfn-lint error: ${lintResult.stderr}`,
      };
    }
    return {
      result: "warning",
      message: `Could not parse cfn-lint output. Exit code: ${lintResult.exitCode}`,
    };
  }
}
