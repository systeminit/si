const getImportExceptions = (resourceId: string, region: string) => ({
  "AWS::EC2::LaunchTemplate": {
    "altCommand": [
      "ec2",
      "describe-launch-template-versions",
      "--launch-template-id",
      resourceId,
      "--versions",
      "$Latest",
      "--region",
      region,
    ],
    "resourceExtractor": "LaunchTemplateVersions.0",
    "fieldMappings": [{
      sourceField: "LaunchTemplateData",
      mappedField: "LaunchTemplateData",
    }, {
      sourceField: "LaunchTemplateName",
      mappedField: "LaunchTemplateName",
    }],
  },
});

async function main({
  thisComponent,
}: Input): Promise<Output> {
  const component = thisComponent.properties;

  let resourceId = _.get(component, ["si", "resourceId"]);

  if (!resourceId) {
    return {
      status: "error",
      message: "No resourceId set, cannot import resource",
    };
  }

  const region = _.get(component, ["domain", "extra", "Region"], "");
  if (!region) {
    return {
      status: "error",
      message: "No region set, cannot import resource",
    };
  }

  const awsResourceType = "AWS::EC2::LaunchTemplate";

  console.log(`Importing ${resourceId} of type ${awsResourceType}`);

  let resourceProperties: Record<string, any> = {};

  const importException =
    getImportExceptions(resourceId, region)[awsResourceType];

  const fieldMappings = importException.fieldMappings;
  const altCommand = importException.altCommand;
  const resourceExtractor = importException.resourceExtractor;

  const child = await siExec.waitUntilEnd("aws", altCommand);

  if (child.exitCode !== 0) {
    console.log(`Failed to import ${awsResourceType}; continuing.`);
    console.log(child.stdout);
    console.log(child.stderr);
    return {
      "status": "error",
      "message": `unable to import: ${child.stderr}`,
    };
  }

  const response = JSON.parse(child.stdout);

  const extractedProperties = _.get(response, resourceExtractor);

  // Explicitly extract mapped fields back into domain
  fieldMappings.forEach(({
    sourceField,
    mappedField,
  }) => {
    console.log(`Importing ${sourceField} as ${mappedField} from response`);
    const value = sourceField
      .split(".")
      .reduce(
        (acc, key) => (acc && acc[key] ? acc[key] : undefined),
        extractedProperties,
      );

    console.log(`Value for ${sourceField} found to be ${value}`);
    if (value !== undefined) {
      resourceProperties[mappedField] = value;
    } else {
      console.log(
        `Value ${value} found to be undefined, ignoring import for this value`,
      );
    }
  });

  const properties = {
    si: {
      resourceId,
    },
    domain: {
      ...resourceProperties,
    },
  };

  return {
    status: "ok",
    message: "Imported Resource",
    ops: {
      update: {
        self: {
          properties,
        },
      },
      actions: {
        self: {
          remove: ["create"],
          add: ["refresh"],
        },
      },
    },
  };
}
