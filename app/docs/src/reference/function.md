# Functions Reference

A Function is code associated with a [schema](./schema.md) which defines an
operation that schema can do.

Functions are written in TypeScript, and executed within a sandbox environment
using [Firecracker](https://firecracker-microvm.github.io/).

## Function Basics

There are 6 types of functions:

- [Action](/reference/actions)
- [Attribute](/reference/attribute)
- [Authentication](/reference/authentication)
- [Code Generation](/reference/code-generation)
- [Management](/reference/management)
- [Qualification](/reference/qualification)

### Executing shell commands

Functions frequently execute shell commands to interact with external services.
Using the [siExec API](/reference/typescript/sandbox/exec/README).

```typescript
const child = siExec.waitUntilEnd("aws", ["ec2", "describe-hosts"]);
```

Would execute the shell command:

```shell
aws ec2 describe-hosts
```

A more complex example from an action:

```typescript
const child = await siExec.waitUntilEnd("aws", [
  "rds",
  "create-db-cluster",
  "--region",
  input?.properties?.domain?.Region || "",
  "--cli-input-json",
  JSON.stringify(code),
]);
```

We're always adding more shell commands to the environment though Nix.
[You can see the current list of included commands in the source code](https://github.com/systeminit/si/blob/main/flake.nix#L96).

Send a PR if you need something added.

### Interacting with HTTP APIs

The [Fetch API](https://developer.mozilla.org/en-US/docs/Web/API/Window/fetch)
is supported.

```typescript
const webpage = await fetch("http://systeminit.com");
```

### Using lodash

The [lodash API](https://lodash.com/docs/4.17.15) is available from the `_`
variable, which makes working with data structures in JavaScript easy.

```typescript
const result = {};
if (component.domain?.Sid) {
  _.set(result, ["Sid"], component.domain.Sid);
}
```

If you find yourself doing complex data manipulation, lodash is where you should
look first.

### Request Storage

When a function has secrets as an input, it runs authentication functions before
it is executed. Information is then passed between functions through the Request
Storage API:

```typescript
requestStorage.getItem("foo");
```

Or to set an item (used only in authentication functions):

```typescript
requestStorage.setItem("foo");
```

## Authoring Functions

<DocTabs tabs="AI Agent,Public API">

<TabPanel value="AI Agent">

To author a function with the AI Agent, simply tell the agent what kind of
function you wish to create on which schema. You may also need to supply the
agent with additional necessary cloud provider documentation for the function to
use.

```prompt
Please add a qualification function to the "example" schema which fails if the "cool" property's value is less than 100.
```

```prompt
⏺ I'll add a qualification function to the "example" schema that fails when the "cool" property is less than 100.

⏺ Perfect! I've added a qualification function to the "example" schema. The function is called "Cool Value Must Be At Least 100" and it:

  - Returns "failure" if the "cool" property value is less than 100
  - Returns "warning" if the "cool" property is not set
  - Returns "success" if the value is 100 or greater

  Now whenever you create a component using the "example" schema, this qualification will automatically run to validate that the "cool" property meets the minimum requirement of 100.
```

</TabPanel>

<TabPanel value="Public API">

To author a function with the Public API, you will need to send a request to the
appropriate API endpoint with your function code, metadata about the function,
and the schema and schema variant ids for the schema variant you want to add the
function to.

These examples create an action function - for other function types, the format
is similar but the endpoint is different.

::: code-group

```typescript [TypeScript]
const response = await schemasApi.createVariantAction({
  workspaceId,
  changeSetId,
  schemaId,
  schemaVariantId,
  createVariantActionFuncV1Request: {
    name, // the name for your function
    description, // a description of the function
    code, // the function code itself
    kind, // the action kind, only necessary for actions
  },
});
```

```python [Python]
request = {
  "name": "function name here",
  "description": "function description here",
  "code": "function code here",
  "kind": "create", # the action kind, only necessary for actions
}

response = schemas_api.create_variant_action(
    workspace_id=workspace_id,
    change_set_id=change_set_id,
    schema_id=schema_id,
    schema_variant_id=schema_variant_id,
    create_variant_action_func_v1_request=request,
)
```

:::

</TabPanel>

</DocTabs>

## Action Function Examples

### Create Action Example

A create action that uses generated code, `siExec` and a secret to create an AWS
EKS cluster:

```typescript
async function main(component: Input): Promise<Output> {
  if (component.properties.resource?.payload) {
    return {
      status: "error",
      message: "Resource already exists",
      payload: component.properties.resource.payload,
    };
  }

  const code = component.properties.code?.["si:genericAwsCreate"]?.code;
  const domain = component.properties?.domain;

  const child = await siExec.waitUntilEnd("aws", [
    "eks",
    "create-cluster",
    "--region",
    domain?.extra?.Region || "",
    "--cli-input-json",
    code || "",
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      status: "error",
      message:
        `Unable to create; AWS CLI exited with non zero code: ${child.exitCode}`,
    };
  }

  const response = JSON.parse(child.stdout).cluster;

  return {
    resourceId: response.name,
    status: "ok",
  };
}
```

### Refresh Action Example

A refresh action example that uses lodash and siExec to update an AWS EKS
cluster:

```typescript
async function main(component: Input): Promise<Output> {
  let name = component.properties?.si?.resourceId;
  const resource = component.properties.resource?.payload;
  if (!name) {
    name = resource.name;
  }
  if (!name) {
    return {
      status: component.properties.resource?.status ?? "error",
      message:
        "Could not refresh, no resourceId present for EKS Cluster component",
    };
  }

  const cliArguments = {};
  _.set(cliArguments, "name", name);

  const child = await siExec.waitUntilEnd("aws", [
    "eks",
    "describe-cluster",
    "--region",
    _.get(component, "properties.domain.extra.Region", ""),
    "--cli-input-json",
    JSON.stringify(cliArguments),
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    if (child.stderr.includes("ResourceNotFoundException")) {
      console.log(
        "EKS Cluster not found upstream (ResourceNotFoundException) so removing the resource",
      );
      return {
        status: "ok",
        payload: null,
      };
    }
    return {
      status: "error",
      payload: resource,
      message:
        `Refresh error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
    };
  }

  const object = JSON.parse(child.stdout).cluster;
  return {
    payload: object,
    status: "ok",
  };
}
```

::: warning

Ensure you include previous resource payload on failure!

:::

### Delete Action Example

A delete action example that uses lodash and siExec:

```typescript
async function main(component: Input): Promise<Output> {
  const cliArguments = {};
  _.set(
    cliArguments,
    "PolicyArn",
    _.get(component, "properties.resource_value.Arn"),
  );

  const child = await siExec.waitUntilEnd("aws", [
    "iam",
    "delete-policy",
    "--cli-input-json",
    JSON.stringify(cliArguments),
  ]);

  if (child.exitCode !== 0) {
    const payload = _.get(component, "properties.resource.payload");
    if (payload) {
      return {
        status: "error",
        payload,
        message:
          `Delete error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
      };
    } else {
      return {
        status: "error",
        message:
          `Delete error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
      };
    }
  }

  return {
    payload: null,
    status: "ok",
  };
}
```

Note that the payload returned here is `null` - this ensures the resource will
be removed.

### Update Action Example

An update action that updates a DigitalOcean Project with new data.

```typescript
async function main(component: Input): Promise<Output> {
  const resource = component.properties.resource?.payload;
  if (!resource) {
    return {
      status: component.properties.resource?.status ?? "error",
      message: "Could not update, no resource present",
    };
  }

  const codeString = component.properties.code?.["DigitalOcean Update Code Gen"]
    ?.code;
  if (!codeString) {
    return {
      status: "error",
      message: "Could not find DigitalOcean Update Code Gen code for resource",
    };
  }

  const token = requestStorage.getEnv("DO_API_TOKEN");
  if (!token) {
    return {
      status: "error",
      message: "DO_API_TOKEN not found (hint: you may need a secret)",
    };
  }

  const endpoint = _.get(
    component.properties,
    ["domain", "extra", "endpoint"],
    "",
  );

  const resourceId = component.properties?.si?.resourceId;

  const updateMethod = _.get(
    component.properties,
    ["domain", "extra", "UpdateMethod"],
    "PUT",
  );

  if (!endpoint) {
    return {
      status: "error",
      message: "No endpoint found in domain configuration",
    };
  }

  if (!resourceId) {
    return {
      status: "error",
      message: "No resource ID found for update",
    };
  }

  // Construct URL - endpoint already starts with /v2/
  let url = `https://api.digitalocean.com${endpoint}/${resourceId}`;

  // Append any required query parameters from metadata
  const requiredQueryParamsJson = _.get(
    component.properties,
    ["domain", "extra", "RequiredQueryParams"],
    "[]",
  );
  const requiredQueryParams = JSON.parse(requiredQueryParamsJson);

  if (requiredQueryParams.length > 0) {
    const queryParts: string[] = [];
    for (const paramName of requiredQueryParams) {
      const paramValue = component.properties?.resource?.payload?.[paramName];
      if (paramValue) {
        queryParts.push(`${paramName}=${encodeURIComponent(paramValue)}`);
      }
    }
    if (queryParts.length > 0) {
      url += `?${queryParts.join("&")}`;
    }
  }

  const response = await fetch(
    url,
    {
      method: updateMethod,
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json",
      },
      body: codeString,
    },
  );

  if (!response.ok) {
    const errorText = await response.text();
    return {
      status: "error",
      message:
        `Unable to update resource; API returned ${response.status} ${response.statusText}: ${errorText}`,
    };
  }

  const responseJson = await response.json();
  const resourceKey = Object.keys(responseJson).find((key) =>
    key !== "links" && key !== "meta"
  );
  const payload = resourceKey ? responseJson[resourceKey] : responseJson;

  return {
    payload,
    status: "ok",
  };
}
```

### Manual Action Example

A manual action that updates the cluster configuration on an AWS EKS cluster,
usking lodash, siExec and the AWS CLI:

```typescript
async function main(component: Input) {
  const resource = component.properties.resource;
  if (!resource) {
    return {
      status: component.properties.resource?.status ?? "ok",
      message: component.properties.resource?.message,
    };
  }

  let json = {
    accessConfig: {
      authenticationMode:
        component.properties.domain.accessConfig.authenticationMode,
    },
    name: resource.name,
  };

  const updateResp = await siExec.waitUntilEnd("aws", [
    "eks",
    "update-cluster-config",
    "--cli-input-json",
    JSON.stringify(json),
    "--region",
    component.properties.domain?.extra.Region || "",
  ]);

  if (updateResp.exitCode !== 0) {
    console.error(updateResp.stderr);
    return {
      status: "error",
      payload: resource,
      message:
        `Unable to update the EKS Cluster Access Config, AWS CLI 2 exited with non zero code: ${updateResp.exitCode}`,
    };
  }

  return {
    payload: resource,
    status: "ok",
  };
}
```

## Attribute Function Examples

The AWS Caller Identity function, which has its output set to `/resource_value`
and takes an input argument called `name` which pulls from `/si/name`:

```typescript
async function main(): Promise<Output> {
  const resp = await siExec.waitUntilEnd("aws", ["sts", "get-caller-identity"]);

  if (resp.exitCode !== 0) {
    console.error(resp.stderr);
    return {
      UserId: "",
      AccountId: "",
      Arn: "",
    };
  }

  const obj = JSON.parse(resp.stdout);

  return {
    UserId: obj.UserId,
    AccountId: obj.Account,
    Arn: obj.Arn,
  };
}
```

This function converts a docker image to a butane systemd unit file. It takes an
input argument named `images`, which pulls from the Input Socket
`Container Image`, and writes to the output location `/domain/systemd/units`:

```typescript
async function main(input: Input): Promise<Output> {
  if (input.images === undefined || input.images === null) return [];
  let images = Array.isArray(input.images) ? input.images : [input.images];

  let units: any[] = [];

  images
    .filter((i: any) => i ?? false)
    .forEach(function (dockerImage: any) {
      // Only allow "valid DNS characters" for the container name, and make sure it doesn't
      // end with a dash character ("-").
      let name = dockerImage.si.name
        .replace(/[^A-Za-z0-9]/g, "-")
        .replace(/-+$/, "")
        .toLowerCase();
      let unit: Record<string, any> = {
        name: name + ".service",
        enabled: true,
      };

      let ports = "";
      let dockerImageExposedPorts = dockerImage.domain.ExposedPorts;
      if (
        !(
          dockerImageExposedPorts === undefined ||
          dockerImageExposedPorts === null
        )
      ) {
        dockerImageExposedPorts.forEach(function (dockerImageExposedPort: any) {
          if (
            !(
              dockerImageExposedPort === undefined ||
              dockerImageExposedPort === null
            )
          ) {
            let parts = dockerImageExposedPort.split("/");
            try {
              // Prefix with a blank space.
              ports = ports + ` --publish ${parts[0]}:${parts[0]}`;
            } catch (err) {}
          }
        });
      }

      let image = dockerImage.domain.image;
      let defaultDockerHost = "docker.io";
      let imageParts = image.split("/");
      if (imageParts.length === 1) {
        image = [defaultDockerHost, "library", imageParts[0]].join("/");
      } else if (imageParts.length === 2) {
        image = [defaultDockerHost, imageParts[0], imageParts[1]].join("/");
      }

      let description = name.charAt(0).toUpperCase() + name.slice(1);

      // Ensure there is no space between "name" and "ports" as ports are optional.
      unit.contents =
        `[Unit]\nDescription=${description}\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nTimeoutStartSec=0\nExecStartPre=-/bin/podman kill ${name}\nExecStartPre=-/bin/podman rm ${name}\nExecStartPre=/bin/podman pull ${image}\nExecStart=/bin/podman run --name ${name}${ports} ${image}\n\n[Install]\nWantedBy=multi-user.target`;

      units.push(unit);
    });

  return units;
}
```

## Authentication Function Examples

The AWS Credential, which supports multiple authentication mechanisms:

```typescript
async function main(secret: Input): Promise<Output> {
  // assume role and set returned creds as env var
  if (secret.AssumeRole) {
    // if they've set keys, use them, otherwise use the si-access-prod profile
    if ((secret.AccessKeyId as string) || (secret.SecretAccessKey as string)) {
      var child = await siExec.waitUntilEnd("aws", [
        "configure",
        "set",
        "aws_access_key_id",
        secret.AcessKeyId as string,
      ]);

      child = await siExec.waitUntilEnd("aws", [
        "configure",
        "set",
        "aws_secret_access_key",
        secret.SecretAccesskey as string,
      ]);

      child = await siExec.waitUntilEnd("aws", [
        "sts",
        "assume-role",
        "--role-arn",
        secret.AssumeRole as string,
        "--role-session-name",
        `SI_AWS_ACCESS_${secret.WorkspaceId}`,
        "--external-id",
        secret.WorkspaceId as string,
      ]);
    } else {
      var child = await siExec.waitUntilEnd("aws", [
        "sts",
        "assume-role",
        "--role-arn",
        secret.AssumeRole as string,
        "--role-session-name",
        `SI_AWS_ACCESS_${secret.WorkspaceId}`,
        "--external-id",
        secret.WorkspaceId as string,
        "--profile",
        "si-access-prod",
      ]);
    }

    if (child.exitCode !== 0) {
      console.error(child.stderr);
      return;
    }

    const creds = JSON.parse(child.stdout).Credentials;

    requestStorage.setEnv("AWS_ACCESS_KEY_ID", creds.AccessKeyId);
    requestStorage.setEnv("AWS_SECRET_ACCESS_KEY", creds.SecretAccessKey);
    requestStorage.setEnv("AWS_SESSION_TOKEN", creds.SessionToken);
  } else {
    requestStorage.setEnv("AWS_ACCESS_KEY_ID", secret.AccessKeyId);
    requestStorage.setEnv("AWS_SECRET_ACCESS_KEY", secret.SecretAccessKey);
    if (secret.SessionToken) {
      requestStorage.setEnv("AWS_SESSION_TOKEN", secret.SessionToken);
    }
  }

  if (secret.Endpoint) {
    requestStorage.setEnv("AWS_ENDPOINT_URL", secret.Endpoint);
  }
}
```

Authenticating with Docker Hub, by writing out a docker configuration json:

```typescript
async function main(secret: Input): Promise<Output> {
  console.log("Starting auth func");
  if (secret.Username && secret.Password) {
    const encoded = Buffer.from(
      `${secret.Username}:${secret.Password}`,
      "utf8",
    ).toString("base64");

    const config: Record<string, any> = {
      auths: {
        "https://index.docker.io/v1/": {
          auth: encoded,
        },
      },
    };

    await siExec.waitUntilEnd("mkdir", ["-p", `${os.homedir()}/.docker`]);

    fs.writeFileSync(
      `${os.homedir()}/.docker/config.json`,
      JSON.stringify(config, null, "\t"),
    );
    console.log(
      `Written credentials file to ${os.homedir()}/.docker/config.json`,
    );
  }
}
```

Using an RDS Database Password, using the setItem API:

```typescript
async function main(secret: Input): Promise<Output> {
  requestStorage.setItem("masterPassword", secret.Password);
}
```

## Code Generation Function Examples

An AWS IAM Role Policy that generates JSON code:

```typescript
async function main(component: Input): Promise<Output> {
  const result = {};
  _.set(result, ["RoleName"], _.get(component, ["domain", "RoleName"]));
  _.set(result, ["PolicyArn"], _.get(component, ["domain", "PolicyArn"]));
  return {
    format: "json",
    code: JSON.stringify(result, null, 2),
  };
}
```

The Butane Ignition code, formatted by an external tool:

```typescript
async function main(input: Input): Promise<Output> {
  const domainJson = JSON.stringify(input.domain);
  domainJson.replace("\n", "\\\\n");
  const options = {
    input: `${domainJson}`,
  };
  const { stdout } = await siExec.waitUntilEnd(
    "butane",
    ["--pretty", "--strict"],
    options,
  );

  return {
    format: "json",
    code: stdout.toString(),
  };
}
```

## Qualification Function Examples

Running the AWS IAM Policy Simulator, based on generated code:

```typescript
async function main(component: Input): Promise<Output> {
  const codeJson = component.code?.["awsIamPolicySimulatorCodeRequest"]
    ?.code as string;

  const args = ["iam", "simulate-custom-policy", "--cli-input-json", codeJson];
  const child = await siExec.waitUntilEnd("aws", args);
  if (child.exitCode !== 0) {
    console.log(child.stdout);
    console.error(child.stderr);
    return {
      result: "failure",
      message:
        `Policy simulator failed; AWS CLI 2 exited with non zero code: ${child.exitCode}`,
    };
  }
  let response = JSON.parse(child.stdout);
  console.log("AWS Policy Response\n");
  console.log(JSON.stringify(response, null, 2));
  let result: "success" | "failure" | "warning" = "success";
  let message = "Policy evaluation succeded";
  for (const res of response["EvaluationResults"]) {
    if (res["EvalDecision"] === "implicitDeny") {
      result = "failure";
      message = "Policy evaluation returned a Deny";
    }
  }

  return {
    result,
    message,
  };
}
```

Ensure butane generates valid ignition:

```typescript
async function main(input: Input): Promise<Output> {
  if (!input.domain) {
    return {
      result: "failure",
      message: "domain is empty",
    };
  }
  const domainJson = JSON.stringify(input.domain);
  // NOTE(nick): this is where one would insert profanities. I'm reformed... right?
  domainJson.replace("\n", "\\\\n");
  const options = {
    input: `${domainJson}`,
  };
  const child = await siExec.waitUntilEnd(
    "butane",
    ["--pretty", "--strict"],
    options,
  );
  return {
    result: child.exitCode === 0 ? "success" : "failure",
    // NOTE(nick): we probably want both stdout and stderr always, but this will suffice for now.
    message: child.exitCode === 0 ? child.stdout : child.stderr,
  };
}
```

Validating that a docker image exists in the registry:

```typescript
async function main(component: Input): Promise<Output> {
  if (!component.domain?.image) {
    return {
      result: "failure",
      message: "no image available",
    };
  }
  const child = await siExec.waitUntilEnd("skopeo", [
    "inspect",
    "--override-os",
    "linux",
    "--override-arch",
    "amd64",
    `docker://${component.domain.image}`,
  ]);
  return {
    result: child.exitCode === 0 ? "success" : "failure",
    message: child.exitCode === 0
      ? "successly found"
      : "docker image not found",
  };
}
```

## Management Function Examples

### Import Function Example

The import function is similar in structure to an action refresh function, but
import works on the component attribute tree rather than the resource. This
means that the functions will change the component in a change set.

```typescript
async function main({ thisComponent }: Input): Promise<Output> {
  const component = thisComponent.properties;

  // 1. Get the resourceId and validate the parameters
  let subnetId = _.get(component, ["si", "resourceId"]);
  if (!subnetId) {
    return {
      status: "error",
      message: "No resourceId set, cannot import Subnet",
    };
  }

  // 2. Get the resource from the upstream API
  const region = _.get(component, ["domain", "Region"]) ?? "";
  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "describe-subnets",
    "--subnet-ids",
    subnetId,
    "--region",
    region,
  ]);

  if (child.exitCode !== 0) {
    console.log(`SubnetId: ${subnetId}`);
    console.error(child.stderr);

    return {
      status: "error",
      message:
        `AWS CLI 2 "aws ec2 describe-subnets" returned non zero exit code(${child.exitCode})`,
    };
  }

  // 3. Parse the response from the upstream API
  const subnets = JSON.parse(child.stdout)?.Subnets;
  const subnet = subnets?.[0];
  if (typeof subnet !== "object") {
    return {
      status: "error",
      message: "No Subnet found in AWS describe-vpcs response",
    };
  }

  // 4. Set the component properties
  component["domain"].AvailabilityZoneId = subnet.AvailabilityZoneId;
  component["domain"].CidrBlock = subnet.CidrBlock;
  component["domain"].AvailabilityZone = subnet.AvailabilityZone;
  component["domain"].VpcId = subnet.VpcId;
  component["domain"].IsPublic = subnet.MapPublicIpOnLaunch ? true : false;

  if (vpc.Tags) {
    const tags: {
      Key: string;
      Value: string;
    }[] = vpc.Tags;
    const newTags: {
      [key: string]: string;
    } = {};
    for (let tag of tags) {
      if (tag.Key === "Name") {
        component["si"]["name"] = tag.Value;
      } else {
        newTags[tag.Key] = tag.Value;
      }
    }
    component["domain"]["tags"] = newTags;
  }

  // 5. Return the updates component and enqueue a refresh function to run on change set merge.
  return {
    status: "ok",
    message: JSON.stringify(subnet),
    ops: {
      update: {
        self: {
          properties: {
            ...component,
          },
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
```

### Template Function Example

A management function that creates components and the connections to them. The
function can use inputs from the connected component and specify the number of
components to create. When creating components, the position of a component is
relative to the position of the management component inside the current view. So
`x: 100, y: 200` will be 100 units to the right and 200 units below the
management component. When updating the component position, the position is the
absolute position of the component.

```typescript
async function main({ thisComponent, components }: Input): Promise<Output> {
  // Access the data in the management function component
  const cidrBlock = thisComponent.properties.domain.BaseCidrBlock;
  const region = thisComponent.properties.domain.Region;
  const tags = thisComponent.properties.domain.Tags;
  const baseName = thisComponent.properties.domain.BaseNamingConvention;

  // Set up an empty map to specify the return value
  const compsToCreate: {
    [key: string]: unknown;
  } = {};

  let counter = 0;

  // Validate the user input
  if (!cidrBlock) {
    return {
      status: "error",
      message: "Cidr Block is missing",
    };
  }

  if (!region) {
    return {
      status: "error",
      message: "Region is missing",
    };
  }

  // Create a VPC component and make it a down frame
  // set it's name, cidrblock, region and tags
  const vpcName = `${baseName}-vpc`;
  if (
    !_.some(
      Object.values(components),
      (comp) => comp.properties.si?.name === vpcName,
    )
  ) {
    compsToCreate[vpcName] = {
      kind: "VPC",
      properties: {
        si: {
          name: vpcName,
          type: "configurationFrameDown",
        },
        domain: {
          CidrBlock: cidrBlock,
          EnableDnsHostnames: true,
          EnableDnsResolution: true,
          tags,
          region,
        },
      },
      geometry: {
        width: 1000,
        height: 1000,
        x: 300,
        y: 300,
      },
    };
    counter++;
  }

  // Create a Public route table
  // Set it's parent to be the VPC
  const publicRouteTableName = `${baseName}-public-route-table`;
  if (
    !_.some(
      Object.values(components),
      (comp) => comp.properties.si?.name === publicRouteTableName,
    )
  ) {
    compsToCreate[publicRouteTableName] = {
      kind: "Route Table",
      properties: {
        si: {
          name: publicRouteTableName,
        },
        domain: {
          Tags: tags,
          Region: region,
        },
      },
      parent: vpcName,
      geometry: {
        x: 600,
        y: 600,
      },
    };
    counter++;
  }

  // Return the list of components to create as part of the function
  // and any message to the user
  return {
    status: "ok",
    message: `Created ${counter} new components`,
    ops: {
      create: compsToCreate,
    },
  };
}
```

### Configuring Existing Components

A component with a management component attached to it can have relationships
with other types of components that it is allowed to manage. These management
relationships are the component context the function can act upon and allow
those components to be configured.

```typescript
async function main({ thisComponent, components }: Input): Promise<Output> {
  // Access the tags from the management function component
  const managedTags = thisComponent.properties?.domain?.Tags;

  let counter = 0;
  const updatedComponents: {
    [key: string]: unknown;
  } = {};

  // Iterate the list of connected components to the management function
  for (let [id, component] of Object.entries(components)) {
    console.log(`Looking at component ${component.properties.si.name}`);
    console.log(`Adding Tags ${managedTags}`);
    if (component.properties.domain.hasOwnProperty("tags")) {
      // Set the updated tags list by merging the current tags
      // and the new tags
      updatedComponents[id] = {
        properties: {
          ...component.properties,
          domain: {
            tags: {
              ...component.properties.domain.tags,
              ...managedTags,
            },
          },
        },
      };
      counter++;
    } else {
      console.log("tags property not found");
    }
  }

  // Return the list of updated components
  return {
    status: "ok",
    message: `Updated ${counter} components`,
    ops: {
      update: updatedComponents,
    },
  };
}
```
