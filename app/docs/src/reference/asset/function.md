# Asset Functions Reference

Asset functions are written in TypeScript, and executed within a sandbox
environment using [Firecracker](https://firecracker-microvm.github.io/).

## Asset Function Basics

There are 6 types of asset functions:

- Action
- Attribute
- Authentication
- Code Generation
- Qualification
- Management

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

## Action Functions

Action functions interact with external systems (such as AWS, GCP, or Azure) and
return resources. They are are en-queued by users in a change set, and executed
when applied to HEAD. The order of execution is determined automatically by
walking the relationships between the components.

There are four types of action function:

1. Functions that create a resource
2. Functions that refresh a resource
3. Functions that delete a resource
4. Manual functions that update or transform a resource

Create, refresh, and delete are automatically en-queued when their relevant
activity is taken on the diagram. Manual functions must be en-queued from the
actions tab of the attribute panel by the user.

### Action function arguments

Action functions take an `Input` argument. It has a `properties` field which
contains an object that has:

- The `si` properties

  These are the core properties set as meta-data for the function. Name, color,
  etc.

- The `domain` properties

  These are the properties specified in the schema itself.

- The `resource` data

  This is the output of the last action, stored as the state of the resource. It
  contains 3 fields:

  - _status_: one of "ok", "warning", or "error"
  - _message_: an optional message
  - _payload_: the resource payload itself

- The `resource_value` data

  This is information pulled into the component properties from resource payload
  data. These are properties added with the `addResourceProp()` method of a
  components schema.

- Any generated `code`

  Generated code is available as a map, whose key is the name of the code
  generation function that generated it.

### Action function return value

Actions return a data structure identical to the resource data above. You should
be careful to always return a payload, even on error - frequently, this is the
last stored payload if it existed.

```typescript
if (input?.properties?.resource?.payload) {
  return {
    status: "error",
    message: "Resource already exists",
    payload: input.properties.resource.payload,
  };
}
```

Remember that `message` is optional:

```typescript
return {
  payload: JSON.parse(child.stdout).DBCluster,
  status: "ok",
};
```

Payload should be returned as a JavaScript object.

### Create action example

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
      message: `Unable to create; AWS CLI exited with non zero code: ${child.exitCode}`,
    };
  }

  const response = JSON.parse(child.stdout).cluster;

  return {
    resourceId: response.name,
    status: "ok",
  };
}
```

### Refresh action example

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
      message: `Refresh error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
    };
  }

  const object = JSON.parse(child.stdout).cluster;
  return {
    payload: object,
    status: "ok",
  };
}
```

:::warning
Ensure you include previous resource payload on failure!
:::

### Delete action example

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
        message: `Delete error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
      };
    } else {
      return {
        status: "error",
        message: `Delete error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
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

### Manual action example

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
      message: `Unable to update the EKS Cluster Access Config, AWS CLI 2 exited with non zero code: ${updateResp.exitCode}`,
    };
  }

  return {
    payload: resource,
    status: "ok",
  };
}
```

## Attribute functions

Attribute functions are used to set properties on components from other properties.

### Attribute function arguments

Actions receive a single `input` object as their argument, whose properties are
determined from the Arguments section of the right-side meta-data panel.

Arguments have a name, which will be used as the property on the `input` object,
and a type, which will be one of the following:

- Any
- Array
- Boolean
- Integer
- JSON
- Map
- Object
- String

These map to their TypeScript equivalents, which also map to the schema property
kinds.

### Attribute function bindings

Each attribute function has a binding, which specifies:

- The output location as a path where this attribute functions return will be
  stored
- A source for each function argument, taken from other Attributes

For example, an attribute function that writes to the `snack` attribute from the
value of the `yummy` attribute would:

- Have a single function argument, `yummy`, whose source is `/root/domain/yummy`
- An output location of `/root/domain/snack`

Bindings can be set from the `Bindings` sub-panel of the functions meta-data.

:::note The UI around setting attribute function bindings is under heavy
development! Hit us up in discord if you have questions. :::

### Attribute function examples

The AWS Caller Identity function, which has its output set to
`/root/resource_value` and takes an input argument called `name` which pulls
from `/root/si/name`:

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
`Container Image`, and writes to the output location
`/root/domain/systemd/units`:

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
      unit.contents = `[Unit]\nDescription=${description}\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nTimeoutStartSec=0\nExecStartPre=-/bin/podman kill ${name}\nExecStartPre=-/bin/podman rm ${name}\nExecStartPre=/bin/podman pull ${image}\nExecStart=/bin/podman run --name ${name}${ports} ${image}\n\n[Install]\nWantedBy=multi-user.target`;

      units.push(unit);
    });

  return units;
}
```

## Authentication functions

Authentication functions are set on assets that define secrets (using the
SecretDefinitionBuilder API). They are then run _before_ any other functions
that require the secret, and either set environment variables or set items on a
local storage to pass information between functions.

Authentication functions return nothing.

### Authentication function arguments

The argument to an authentication function is a secret, which maps to the
property definitions from the SecretDefinitionBuilder.

### The requestStorage API

Authentication functions make use of the requestStorage API. It allows you to:

- Set environment variables with `setEnv`
- Get environment variables with `getEnv`
- Store a javascript object as an item by key with `setItem`
- Get items by their key with `getItem`
- Check for the existence of an environment key with `getEnvKey` or an item with
  `getKeys`

### Authentication function examples

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

## Code Generation functions

Code Generation functions generate code from the component data. The results
show up in the Code tab in the attribute panel, and can be accessed in action
functions or attribute functions by their function name (in a map).

### Code Generation function arguments

Code Generation functions take a single argument, `component`, which has 3
possible properties:

- `domain`, which has the domain properties of the component
- `resource`, which has the resource information
- `deleted_at`, a string with the time of a deletion

### Code Generation function return value

The return value for a code generation function is a string representing the
format of the data, and a string for the generated code:

```typescript
{
    format: "json",
    code: '{ "poop": "canoe" }',
}
```

### Code Generation function examples

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

## Qualification functions

Qualification functions take in information about the component, resource, and
generated code, and use it to validate the component.

### Qualification function arguments

Qualification functions take an argument, `component`, which has:

- `code`, available as a map of code generation results keyed on function name
- `domain`, which has the domain properties of the component
- `resource`, which has the resource information
- `deleted_at`, a string with the time of a deletion

### Qualification function return value

Qualification functions return a result, which is one of `success`, `warning`,
or `failure`, along with a message explaining the result.

```typescript
return {
    result: "success",
    message: "it worked!',
}
```

### Qualification function examples

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
      message: `Policy simulator failed; AWS CLI 2 exited with non zero code: ${child.exitCode}`,
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
    message:
      child.exitCode === 0 ? "successly found" : "docker image not found",
  };
}
```

## Management Functions

A management function allows the creation, configuration, and management of
other components and connections on the graph. They are executed in a change set
and can enqueue actions to run when the change set is applied to HEAD.

Management functions are typically used to achieve the following things:

1. Importing existing resources into System Initiative
2. Creation of a template
3. Configuring pre-existing components

These functions will also be able to be used for a host of other types of
use-cases, like:

- Running analyses across a segment of components
- Manage infrastructure across multiple environments

### Management function arguments

Management functions take an `Input` argument. This argument is an object that
contains:

- `currentView`

  This is the view in which the management function will execute. This defaults
  to the `DEFAULT` view on the diagram.

- `thisComponent`

  This is the represention of the component to which the management function is
  currently running from. In this argument, is the `properties` object, and that
  will expose `si`, `domain`, and `resource` properties as well as the `geometry`
  of the current component. The geometry is the `height`, `width`, `x`, and `y`
  coordinates for the component.

- `components`

  This object contains all of the components that a management
  function is connected to, keyed by the component id. Each of these components
  exposes the component type, which is essentially the schema name, the `properties`,
  and `geometry`.

The entire structure of the input is:

```typescript
type Input = {
  currentView: string;
  thisComponent: {
    properties: {
      si?: {
        name?: string | null;
        protected?: boolean | null;
        type?: string | null;
        color?: string | null;
        resourceId?: string | null;
      } | null;
      domain?: {
        // Properties based on the type of the component function is attached to
        // ...
      } | null;
      secrets?: {
        credential?: string | null;
      } | null;
      resource?: {
        status?: "ok" | "warning" | "error" | undefined | null;
        message?: string | null;
        payload?: any | null;
        last_synced?: string | null;
      } | null;
      resource_value?: {} | null;
      code?: Record<
        string,
        {
          code?: string | null;
          format?: string | null;
        }
      > | null;
      qualification?: Record<
        string,
        {
          result?: string | null;
          message?: string | null;
        }
      > | null;
      deleted_at?: string | null;
    };
    geometry: { [key: string]: Geometry };
  };
  components: {
    [key: string]: {
      kind: string;
      properties?: {
        si?: {
          name?: string | null;
          protected?: boolean | null;
          type?: string | null;
          color?: string | null;
          resourceId?: string | null;
        } | null;
        domain?: {
          // Properties based on the type of the component being managed
          // ...
        } | null;
        secrets?: {
          credential?: string | null;
        } | null;
        resource?: {
          status?: "ok" | "warning" | "error" | undefined | null;
          message?: string | null;
          payload?: any | null;
          last_synced?: string | null;
        } | null;
        resource_value?: {} | null;
        code?: Record<
          string,
          {
            code?: string | null;
            format?: string | null;
          }
        > | null;
        qualification?: Record<
          string,
          {
            result?: string | null;
            message?: string | null;
          }
        > | null;
        deleted_at?: string | null;
      };
      geometry?: { [key: string]: Geometry };
    };
  };
};
```

where the `geometry` object is:

```typescript
type Geometry = {
  x?: number;
  y?: number;
  width?: number;
  height?: number;
};
```

### Management function return type

The return type for the management function is as follows:

```typescript
type Output = {
  status: "ok" | "error";
  ops?: {
    create?: {
      [key: string]: {
        kind: string;
        properties?: {
          si?: {
            name?: string | null;
            protected?: boolean | null;
            type?: string | null;
            color?: string | null;
            resourceId?: string | null;
          } | null;
          domain?: {
            // Properties based on the type of the component being managed
            // ...
          } | null;
          secrets?: {
            credential?: string | null;
          } | null;
          resource?: {
            status?: "ok" | "warning" | "error" | undefined | null;
            message?: string | null;
            payload?: any | null;
            last_synced?: string | null;
          } | null;
          resource_value?: {} | null;
          code?: Record<
            string,
            {
              code?: string | null;
              format?: string | null;
            }
          > | null;
          qualification?: Record<
            string,
            {
              result?: string | null;
              message?: string | null;
            }
          > | null;
          deleted_at?: string | null;
        };
        geometry?: Geometry;
      };
    };
    update?: {
      [key: string]: {
        properties?: { [key: string]: unknown };
        geometry?: { [key: string]: Geometry };
      };
    };
    actions?: {
      [key: string]: {
        add?: ("create" | "update" | "refresh" | "delete" | string)[];
        remove?: ("create" | "update" | "refresh" | "delete" | string)[];
      };
    };
  };
  message?: string | null;
};
```

### Import function example

The import function is similar in structure to an action refresh function, 
but import works on the component attribute tree rather than the resource. This
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
      message: `AWS CLI 2 "aws ec2 describe-subnets" returned non zero exit code(${child.exitCode})`,
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

### Template function example

A management function that creates components and the connections to them. The
function can use inputs from the connected asset and specify the number
of components to create. When creating components, the position of a component
is relative to the position of the management component inside the current view.
So `x: 100, y: 200` will be 100 units to the right and 200 units below the
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
  // Set its parent to be the VPC
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

### Configuring pre-existing components

An asset with a management component attached to it can have
`management edges` to other types of components that it is allowed to manage.
These management edges are the component context the function can act upon
and allow those components to be configured.

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

## Using the si-generator to generate functions for AWS Services

The [System Initiative source code](https://github.com/systeminit/si) repository
contains a program that will automatically generate schema for AWS services.
Check out the repository and navigate to the `bin/si-generator` directory.

Ensure you have the [aws cli](https://aws.amazon.com/cli/) installed.

The generator can create Action functions, and a standardized Code Generation
function for AWS services.

### Create actions

Start by finding the action you want to model. For example, to model the
deleting of an AWS EC2 Key Pair, the command would be `aws ec2 create-key-pair`.

```shell
$ deno run ./main.ts create ec2 create-key-pair
```

Which results in the following action function:

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
    "ec2",
    "create-key-pair",
    "--region",
    domain?.extra?.Region || "",
    "--cli-input-json",
    code || "",
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      status: "error",
      message: `Unable to create; AWS CLI 2 exited with non zero code: ${child.exitCode}`,
    };
  }

  const response = JSON.parse(child.stdout);

  return {
    payload: response,
    status: "ok",
  };
}
```

### Delete Actions

Start by finding the action you want to model. For example, to model the
deleting of an AWS EC2 Key Pair, the command would be `aws ec2 delete-key-pair`.

First, see the input skeleton to the call:

```shell
$ aws ec2 delete-key-pair --generate-cli-skeleton
{
    "KeyName": "",
    "KeyPairId": "",
    "DryRun": true
}
```

Isolate the input path for the call - in this case, it is `KeyName`.

Then, find the correct path for the domain property you want to use as the
argument as it would be specified in the Action function - in this case, it is
`properties.domain.KeyName`.

Then run the generator:

```shell
$ deno run ./main.ts delete ec2 delete-key-pair --input KeyName:properties.domain.KeyName
```

Which results in the following action function:

```typescript
async function main(component: Input): Promise<Output> {
  const cliArguments = {};
  _.set(cliArguments, "KeyName", _.get(component, "properties.domain.KeyName"));

  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "delete-key-pair",
    "--region",
    _.get(component, "properties.domain.extra.Region", ""),
    "--cli-input-json",
    JSON.stringify(cliArguments),
  ]);

  if (child.exitCode !== 0) {
    const payload = _.get(component, "properties.resource.payload");
    if (payload) {
      return {
        status: "error",
        payload,
        message: `Delete error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
      };
    } else {
      return {
        status: "error",
        message: `Delete error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
      };
    }
  }

  return {
    payload: null,
    status: "ok",
  };
}
```

### Refresh functions

Start by finding the action you want to model. For example, to model the
refreshing of an IAM Instance Profile, the command would be
`aws iam get-instance-profile`.

First, see the input skeleton to the call:

```shell
$ aws iam get-instance-profile --generate-cli-skeleton
{
    "InstanceProfileName": ""
}
```

Isolate the input path for the call - in this case, it is `InstanceProfileName`.

Then, find the correct path for the domain property you want to use as the
argument as it would be specified in the Action function - in this case, it is
`properties.domain.InstanceProfileName`.

Examine the output of a manual call to the CLI to understand the
output data:

```json
{
  "InstanceProfile": {
    "InstanceProfileId": "AID2MAB8DPLSRHEXAMPLE",
    "Roles": [
      {
        "AssumeRolePolicyDocument": "<URL-encoded-JSON>",
        "RoleId": "AIDGPMS9RO4H3FEXAMPLE",
        "CreateDate": "2013-01-09T06:33:26Z",
        "RoleName": "Test-Role",
        "Path": "/",
        "Arn": "arn:aws:iam::336924118301:role/Test-Role"
      }
    ],
    "CreateDate": "2013-06-12T23:52:02Z",
    "InstanceProfileName": "ExampleInstanceProfile",
    "Path": "/",
    "Arn": "arn:aws:iam::336924118301:instance-profile/ExampleInstanceProfile"
  }
}
```

The output path for the resource data is under the `InstanceProfile` key.

Then run the generator:

```shell
$ deno run ./main.ts refresh iam get-instance-profile --input InstanceProfileName:properties.domain.InstanceProfileName --output InstanceProfile
```

Which generates the following refresh function:

```typescript
async function main(component: Input): Promise<Output> {
  const cliArguments = {};
  _.set(
    cliArguments,
    "InstanceProfileName",
    _.get(component, "properties.domain.InstanceProfileName"),
  );

  const child = await siExec.waitUntilEnd("aws", [
    "iam",
    "get-instance-profile",
    "--region",
    _.get(component, "properties.domain.extra.Region", ""),
    "--cli-input-json",
    JSON.stringify(cliArguments),
  ]);

  if (child.exitCode !== 0) {
    const payload = _.get(component, "properties.resource.payload");

    if (payload) {
      return {
        status: "error",
        payload,
        message: `Refresh error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
      };
    } else {
      return {
        status: "error",
        message: `Refresh error; exit code ${child.exitCode}.\n\nSTDOUT:\n\n${child.stdout}\n\nSTDERR:\n\n${child.stderr}`,
      };
    }
  }

  const response = JSON.parse(child.stdout);
  const resource = {};
  _.merge(resource, _.get(response, "InstanceProfile"));
  if (!resource) {
    return {
      status: "error",
      message: `Resource not found in payload.\n\nResponse:\n\n${child.stdout}`,
    };
  }
  return {
    payload: resource,
    status: "ok",
  };
}
```
