import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { validateFunctionCode } from "../validators/funcValidator.ts";
import { SchemasApi, FuncsApi } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";

const name = "func-create-or-edit";
const title = "Create or update a function for an existing schema.";
const description = `
<description>
Create or update a function for an existing schema following the usage workflow.
</description>
<usage-workflow>
  *ALWAYS* follow this workflow:
  1. VALIDATION PHASE (REQUIRED):
     - If the user provides functionCode, you *MUST* first analyze it against the requested
  functionType and check that it is valid TypeScript code.
     - Check the function signature matches the expected type (see examples in functionCode
  parameter)
     - Verify return types match: qualification returns {result, message}, codegen returns
  {format, code}, etc.
     - If code doesn't match the requested type, STOP and inform the user of the mismatch
     - NEVER proceed with tool execution if code validation fails
  2. LOOKUP PHASE:
     - If user only gives schema name, use schema-find tool to get schemaId
     - If user doesn't specify functionType, ask them which type
     - If functionType is "action" but no actionKind given, ask which kind
  3. EXECUTION PHASE:
     - Only after validation passes, call this tool
     - Do not mention schemaVariantId to the user
     - Do not mention locking/unlocking schemas to the user
  CRITICAL: Step 1 is mandatory when functionCode is provided.
  <validation-checklist>
    Before calling this tool, confirm:
    [ ] I have analyzed the functionCode syntax
    [ ] I have verified the function signature matches the functionType
    [ ] I have verified the return type matches the functionType
    [ ] If validation failed, I have stopped and informed the user
  </validation-checklist>
</usage-workflow>`;

const DEFAULT_QUALIFICATION_FUNCTION = `function main() {
    return { result: "success", message: "All good!" };
}`;

const DEFAULT_CODEGEN_FUNCTION = `async function main(component: Input): Promise<Output> {
  return {
    format: "json",
    code: JSON.stringify(component),
  };
}`;

const DEFAULT_MANAGEMENT_FUNCTION = `async function main({ thisComponent, components }: Input): Promise<Output> {
  throw new Error("unimplemented!");
}`;

const DEFAULT_ACTION_FUNCTION = `async function main(component: Input): Promise<Output> {
  throw new Error("unimplemented!");
}`;

// The biggest piece of context here is for validating functions!
// TODO - this may be a good spot to cut context if we need to
const functionCodeDescribe = [
  `<description>
    A typescript function definition. Documentation on functions can be found at https://docs.systeminit.com/reference/asset/function
    *Always* follow the workflow outlined in the <usage-workflow>.
  </description>`,
  `<important-instructions-you-should-always-follow-first>
  Before submitting code to this tool, please ensure that the function code is valid TypeScript and follows the guidelines provided in the documentation for the given function type.
  If the function code looks like a different function type than the one asked for, STOP and tell the user and ask them what to do.
  It is more important to ensure that the function code is valid than to complete the users instructions quickly.
  Always follow the workflow in the usage instructions!
  </important-instructions-you-should-always-follow-first>`,
  `<qualification-explanation>
  Qualification functions take an argument, component, which has:
  code, available as a map of code generation results keyed on function name
  domain, which has the domain properties of the component
  resource, which has the resource information
  deleted_at, a string with the time of a deletion
  </qualification-explanation>`,
  `<qualification-good-example>
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
        message: "Policy simulator failed; AWS CLI 2 exited with non zero code",
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
  </qualification-good-example>`,
  `<codegen-explanation>
  Code Generation functions take a single argument, component, which has 3 possible properties:
  domain, which has the domain properties of the component
  resource, which has the resource information
  deleted_at, a string with the time of a deletion
  </codegen-explanation>`,
  `<codegen-good-example>
  async function main(component: Input): Promise<Output> {
    const result = {};
    _.set(result, ["RoleName"], _.get(component, ["domain", "RoleName"]));
    _.set(result, ["PolicyArn"], _.get(component, ["domain", "PolicyArn"]));
    return {
      format: "json",
      code: JSON.stringify(result, null, 2),
    };
  }
  </codegen-good-example>`,
  `<management-explanation>
  Management functions take an Input argument. This argument is an object that contains:
  currentView - This is the view in which the management function will execute. This defaults to the DEFAULT view on the diagram.
  thisComponent - This is the represention of the component to which the management function is currently running from. In this argument, is the properties object, and that will expose si, domain, and resource properties.
  components - This object contains all of the components that a management function is connected to, keyed by the component id. Each of these components exposes the component type, which is essentially the schema name, the properties, parent and, an array connections.
  </management-explanation>`,
  `<management-good-example>
  import {
      kebabCase
  } from "jsr:@mesqueeb/case-anything";
  async function main({
      thisComponent
  }: Input): Promise < Output > {
      const vars = thisComponent.properties.domain
      console.log(vars)
      const region = _.get(thisComponent, ["properties", "domain", "extra", "Region"]);
      if (!region) {
          throw new Error(
              'Missing required property: "Region". Please set your Region property to run this AWS VPC Template.'
          );
      }
      const specs: Output["ops"]["create"][string][] = [];
      const cidrBlockRaw = vars["CidrBlock"];
      const normalizedCidr = cidrBlockRaw.includes("/") ? cidrBlockRaw : \`\${cidrBlockRaw}/16\`;
      const vpcSpec: Output["ops"]["create"][string] = {
          kind: "AWS::EC2::VPC",
          properties: {
              si: {
                  name: kebabCase(vars["VPC Name"]) + "-vpc",
              },
              domain: {
                  CidrBlock: normalizedCidr,
                  EnableDnsHostnames: vars["Enable DNS Hostnames"],
                  EnableDnsSupport: vars["Enable DNS Resolution"],
                  InstanceTenancy: vars["Tenancy"],
              },
          },
          attributes: {
              "/domain/extra/Region": {
                  $source: thisComponent.sources["/domain/extra/Region"],
              },
              "/secrets/AWS Credential": {
                  $source: thisComponent.sources["/secrets/AWS Credential"],
              },
          },
      };
      specs.push(vpcSpec);
      const igwAttachSpec: Output["ops"]["create"][string] = {
          kind: "AWS::EC2::VPCGatewayAttachment",
          properties: {
              si: {
                  name: kebabCase(vars["VPC Name"]) + "-igw-attach",
              },
          },
          attributes: {
              "/domain/extra/Region": {
                  $source: thisComponent.sources["/domain/extra/Region"],
              },
              "/secrets/AWS Credential": {
                  $source: thisComponent.sources["/secrets/AWS Credential"],
              },
              "/domain/VpcId": {
                  $source: {
                      component: vpcSpec.properties.si.name,
                      path: "/resource_value/VpcId",
                  },
              },
          },
      };
      specs.push(igwAttachSpec);
      const routeInternetSpec: Output["ops"]["create"][string] = {
          kind: "AWS::EC2::Route",
          properties: {
              si: {
                  name: kebabCase(vars["VPC Name"]) + "-route-internet",
              },
              domain: {
                  DestinationCidrBlock: "0.0.0.0/0",
              },
          },
          attributes: {
              "/domain/extra/Region": {
                  $source: thisComponent.sources["/domain/extra/Region"],
              },
              "/secrets/AWS Credential": {
                  $source: thisComponent.sources["/secrets/AWS Credential"],
              },
              },
          },
      };
      specs.push(routeInternetSpec);
      const numberOfAzs = _.toNumber(vars["Number of Availability Zones (AZs)"]);
      for (let x = 0; x < numberOfAzs; x++) {
          const subnetCount = x + 1;
          let azName = thisComponent.properties.domain.extra.Region;
          if (x === 0) {
              azName = \`\${azName}a\`;
          } else if (x === 1) {
              azName = \`\${azName}b\`;
          } else if (x === 2) {
              azName = \`\${azName}c\`;
          }
          const publicSubnets = [];
          if (vars["Public Subnets"]) {
              let cidrBlock = "";
              const ipParts = vars["CidrBlock"].split(".");
              if (x === 0) {
                  cidrBlock = \`\${ipParts[0]}.\${ipParts[1]}.32.0/20\`;
              } else if (x === 1) {
                  cidrBlock = \`\${ipParts[0]}.\${ipParts[1]}.96.0/20\`;
              } else if (x === 2) {
                  cidrBlock = \`\${ipParts[0]}.\${ipParts[1]}.160.0/20\`;
              }
              const subnetPublicSpec: Output["ops"]["create"][string] = {
                  kind: "AWS::EC2::Subnet",
                  properties: {
                      si: {
                          name: kebabCase(vars["VPC Name"]) + "-subnet-pub-" + subnetCount,
                      },
                      domain: {
                          AvailabilityZone: azName,
                          CidrBlock: cidrBlock,
                          MapPublicIpOnLaunch: true,
                      },
                  },
                  attributes: {
                      "/domain/extra/Region": {
                          $source: thisComponent.sources["/domain/extra/Region"],
                      },
                      "/secrets/AWS Credential": {
                          $source: thisComponent.sources["/secrets/AWS Credential"],
                      },
                      "/domain/VpcId": {
                          $source: {
                              component: vpcSpec.properties.si.name,
                              path: "/resource_value/VpcId",
                          },
                      },
                  },
              };
              specs.push(subnetPublicSpec);
              publicSubnets[x] = {
                  spec: subnetPublicSpec,
              };
              const rtbaSpec: Output["ops"]["create"][string] = {
                  kind: "AWS::EC2::SubnetRouteTableAssociation",
                  properties: {
                      si: {
                          name: kebabCase(vars["VPC Name"]) + "-srtba-public-" + subnetCount,
                      },
                  },
                  attributes: {
                      "/domain/extra/Region": {
                          $source: thisComponent.sources["/domain/extra/Region"],
                      },
                      "/secrets/AWS Credential": {
                          $source: thisComponent.sources["/secrets/AWS Credential"],
                      },
                      "/domain/SubnetId": {
                          $source: {
                              component: subnetPublicSpec.properties.si.name,
                              path: "/resource_value/SubnetId",
                          },
                      },
                  },
              };
              specs.push(rtbaSpec);
          }
          if (vars["Private Subnets"]) {
              let cidrBlock = "";
              const ipParts = vars["CidrBlock"].split(".");
              if (x === 0) {
                  cidrBlock = \`\${ipParts[0]}.\${ipParts[1]}.0.0/19\`;
              } else if (x === 1) {
                  cidrBlock = \`\${ipParts[0]}.\${ipParts[1]}.64.0/19\`;
              } else if (x === 2) {
                  cidrBlock = \`\${ipParts[0]}.\${ipParts[1]}.128.0/19\`;
              }
              const subnetPrivateSpec: Output["ops"]["create"][string] = {
                  kind: "AWS::EC2::Subnet",
                  properties: {
                      si: {
                          name: kebabCase(vars["VPC Name"]) + "-subnet-priv-" + subnetCount,
                      },
                      domain: {
                          AvailabilityZone: azName,
                          CidrBlock: cidrBlock,
                      },
                  },
                  attributes: {
                      "/domain/extra/Region": {
                          $source: thisComponent.sources["/domain/extra/Region"],
                      },
                      "/secrets/AWS Credential": {
                          $source: thisComponent.sources["/secrets/AWS Credential"],
                      },
                      "/domain/VpcId": {
                          $source: {
                              component: vpcSpec.properties.si.name,
                              path: "/resource_value/VpcId",
                          },
                      },
                  },
              };
              specs.push(subnetPrivateSpec);
              if (vars["NAT Gateways"]) {
                  const eipSpec: Output["ops"]["create"][string] = {
                      kind: "AWS::EC2::EIP",
                      properties: {
                          si: {
                              name: kebabCase(vars["VPC Name"]) + "-eip-ngw-" + subnetCount,
                          },
                      },
                      attributes: {
                          "/domain/extra/Region": {
                              $source: thisComponent.sources["/domain/extra/Region"],
                          },
                          "/secrets/AWS Credential": {
                              $source: thisComponent.sources["/secrets/AWS Credential"],
                          },
                      },
                  };
                  specs.push(eipSpec);
                  const ngwSpec: Output["ops"]["create"][string] = {
                      kind: "AWS::EC2::NatGateway",
                      properties: {
                          si: {
                              name: kebabCase(vars["VPC Name"]) + "-ngw-" + subnetCount,
                          },
                      },
                      attributes: {
                          "/domain/extra/Region": {
                              $source: thisComponent.sources["/domain/extra/Region"],
                          },
                          "/secrets/AWS Credential": {
                              $source: thisComponent.sources["/secrets/AWS Credential"],
                          },
                          "/domain/AllocationId": {
                              $source: {
                                  component: eipSpec.properties.si.name,
                                  path: "/resource_value/AllocationId",
                              },
                          },
                          "/domain/SubnetId": {
                              $source: {
                                  component: publicSubnets[x].spec.properties.si.name,
                                  path: "/resource_value/SubnetId",
                              },
                          },
                      },
                  };
                  specs.push(ngwSpec);
                  const privateRtbSpec: Output["ops"]["create"][string] = {
                      kind: "AWS::EC2::RouteTable",
                      properties: {
                          si: {
                              name: kebabCase(vars["VPC Name"]) + "-rtb-private-" + subnetCount,
                          },
                      },
                      attributes: {
                          "/domain/extra/Region": {
                              $source: thisComponent.sources["/domain/extra/Region"],
                          },
                          "/secrets/AWS Credential": {
                              $source: thisComponent.sources["/secrets/AWS Credential"],
                          },
                          "/domain/VpcId": {
                              $source: {
                                  component: vpcSpec.properties.si.name,
                                  path: "/resource_value/VpcId",
                              },
                          },
                      },
                  };
                  specs.push(privateRtbSpec);
                  const rtbaSpec: Output["ops"]["create"][string] = {
                      kind: "AWS::EC2::SubnetRouteTableAssociation",
                      properties: {
                          si: {
                              name: kebabCase(vars["VPC Name"]) + "-srtba-private-" + subnetCount,
                          },
                      },
                      attributes: {
                          "/domain/extra/Region": {
                              $source: thisComponent.sources["/domain/extra/Region"],
                          },
                          "/secrets/AWS Credential": {
                              $source: thisComponent.sources["/secrets/AWS Credential"],
                          },
                          "/domain/SubnetId": {
                              $source: {
                                  component: subnetPrivateSpec.properties.si.name,
                                  path: "/resource_value/SubnetId",
                              },
                          },
                          "/domain/RouteTableId": {
                              $source: {
                                  component: privateRtbSpec.properties.si.name,
                                  path: "/resource_value/RouteTableId",
                              },
                          },
                      },
                  };
                  specs.push(rtbaSpec);
                  const privateRouteInternetSpec: Output["ops"]["create"][string] = {
                      kind: "AWS::EC2::Route",
                      properties: {
                          si: {
                              name: kebabCase(vars["VPC Name"]) + "-route-internet-private" + subnetCount,
                          },
                          domain: {
                              DestinationCidrBlock: "0.0.0.0/0",
                          },
                      },
                      attributes: {
                          "/domain/extra/Region": {
                              $source: thisComponent.sources["/domain/extra/Region"],
                          },
                          "/secrets/AWS Credential": {
                              $source: thisComponent.sources["/secrets/AWS Credential"],
                          },
                          "/domain/NatGatewayId": {
                              $source: {
                                  component: ngwSpec.properties.si.name,
                                  path: "/resource_value/NatGatewayId",
                              },
                          },
                          "/domain/RouteTableId": {
                              $source: {
                                  component: privateRtbSpec.properties.si.name,
                                  path: "/resource_value/RouteTableId",
                              },
                          },
                      },
                  };
                  specs.push(privateRouteInternetSpec);
              }
          }
      }
      // Check for duplicate si names in the abscene of component idempotency keys
      const seenNames = new Set < string > ();
      for (const spec of specs) {
          const name = _.get(spec, ["properties", "si", "name"]);
          if (seenNames.has(name)) {
              throw new Error(
                  \`Duplicate properties.si.name found: "\${name}", please regenerate the template after fixing the duplicate names or modify the id references in the management function\`,
              );
          }
          seenNames.add(name);
      }
      return {
          status: "ok",
          ops: {
              create: Object.fromEntries(
                  specs.map((spec) => {
                      const name = spec.properties?.si?.name;
                      if (!name) {
                          throw new Error("Missing properties.si.name on a spec");
                      }
                      return [name, spec];
                  })
              ),
          },
      };
  }
  </management-good-example>`,
  `<action-explanation>
  Action functions interact with external systems and return resources.
  Each schema can only have one of each type of action function except Manual. Any number of Manual actions are allowed.
  There are five types of action functions - Create, Destroy, Refresh, Update, and Manual
  </action-explanation>`,
  `<action-create-example>
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
        message: \`Unable to create; AWS CLI exited with non zero code: \${child.exitCode}\`,
      };
    }

    const response = JSON.parse(child.stdout).cluster;

    return {
      resourceId: response.name,
      status: "ok",
    };
  }
  </action-create-example>`,
  `<action-destroy-example>
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
          message: \`Delete error; exit code \${child.exitCode}.\n\nSTDOUT:\n\n\${child.stdout}\n\nSTDERR:\n\n\${child.stderr}\`,
        };
      } else {
        return {
          status: "error",
          message: \`Delete error; exit code \${child.exitCode}.\n\nSTDOUT:\n\n\${child.stdout}\n\nSTDERR:\n\n\${child.stderr}\`,
        };
      }
    }

    return {
      payload: null,
      status: "ok",
    };
  }
  </action-destroy-example>`,
  `<action-refresh-example>
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
        message: \`Refresh error; exit code \${child.exitCode}.\n\nSTDOUT:\n\n\${child.stdout}\n\nSTDERR:\n\n\${child.stderr}\`,
      };
    }

    const object = JSON.parse(child.stdout).cluster;
    return {
      payload: object,
      status: "ok",
    };
  }
  </action-refresh-example>`,
  `<action-update-example>
  async function main(component: Input): Promise<Output> {
    if (!component.properties.resource?.payload) {
      return {
        status: "error",
        message: "Unable to queue an update action on a component without a resource",
      };
    }

    let resourceId = component.properties?.si?.resourceId;

    const refreshChild = await siExec.waitUntilEnd("aws", [
      "cloudcontrol",
      "get-resource",
      "--region",
      _.get(component, "properties.domain.extra.Region", ""),
      "--type-name",
      _.get(component, "properties.domain.extra.AwsResourceType", ""),
      "--identifier",
      resourceId,
    ]);

    if (refreshChild.exitCode !== 0) {
      console.log("Failed to refresh cloud control resource");
      console.log(refreshChild.stdout);
      console.error(refreshChild.stderr);
      return {
        status: "error",
        message:
          \`Update error while fetching current state; exit code \${refreshChild.exitCode}.\n\nSTDOUT:\n\n\${refreshChild.stdout}\n\nSTDERR:\n\n\${refreshChild.stderr}\`,
      };
    }

    const resourceResponse = JSON.parse(refreshChild.stdout);
    const currentState = JSON.parse(
      resourceResponse["ResourceDescription"]["Properties"],
    );

    const desiredProps = JSON.parse(
      component.properties.code?.["awsCloudControlUpdate"]?.code,
    )?.DesiredState;

    // Copy secrets to desired props
    const propUsageMap = JSON.parse(
      component.properties?.domain.extra.PropUsageMap,
    );

    addSecretsToPayload(desiredProps, propUsageMap);

    const desiredState = _.cloneDeep(currentState);
    _.merge(desiredState, desiredProps);
    let patch;
    try {
      patch = jsonpatch.compare(currentState, desiredState, true);
    } catch (e) {
      return {
        status: "error",
        message: \`jsonpatch error\n\nMessage: \${e}\`,
      };
    }
    console.log("Computed patch", patch);

    const child = await siExec.waitUntilEnd("aws", [
      "cloudcontrol",
      "update-resource",
      "--region",
      _.get(component, "properties.domain.extra.Region", ""),
      "--type-name",
      _.get(component, "properties.domain.extra.AwsResourceType", ""),
      "--identifier",
      resourceId,
      "--patch-document",
      JSON.stringify(patch),
    ]);

    if (child.exitCode !== 0) {
      console.error(child.stderr);
      return {
        status: "error",
        message:
          \`Unable to update; AWS CLI 2 exited with non zero code: \${child.exitCode}\`,
      };
    }

    const progressEvent = JSON.parse(child.stdout);
    console.log("Progress Event", progressEvent);

    const delay = (time: number) => {
      return new Promise((res) => {
        setTimeout(res, time);
      });
    };

    let finished = false;
    let success = false;
    let wait = 1000;
    const upperLimit = 10000;
    let message = "";
    let identifier = "";

    while (!finished) {
      const child = await siExec.waitUntilEnd("aws", [
        "cloudcontrol",
        "get-resource-request-status",
        "--region",
        _.get(component, "properties.domain.extra.Region", ""),
        "--request-token",
        _.get(progressEvent, ["ProgressEvent", "RequestToken"]),
      ]);

      if (child.exitCode !== 0) {
        console.error(child.stderr);
        return {
          status: "error",
          message:
            \`Unable to create; AWS CLI 2 exited with non zero code: \${child.exitCode}\`,
        };
      }
      const currentProgressEvent = JSON.parse(child.stdout);
      console.log("Current Progress", currentProgressEvent);
      const operationStatus =
        currentProgressEvent["ProgressEvent"]["OperationStatus"];
      if (operationStatus == "SUCCESS") {
        finished = true;
        success = true;
        identifier = currentProgressEvent["ProgressEvent"]["Identifier"];
      } else if (operationStatus == "FAILED") {
        finished = true;
        success = false;
        message = currentProgressEvent["ProgressEvent"]["StatusMessage"] ||
          currentProgressEvent["ProgressEvent"]["ErrorCode"];
      } else if (operationStatus == "CANCEL_COMPLETE") {
        finished = true;
        success = false;
        message = "Operation Canceled by API or AWS.";
      }

      if (!finished) {
        console.log("\nWaiting to check status!\n");
        await delay(wait);
        if (wait != upperLimit) {
          wait = wait + 1000;
        }
      }
    }

    if (success) {
      const child = await siExec.waitUntilEnd("aws", [
        "cloudcontrol",
        "get-resource",
        "--region",
        _.get(component, "properties.domain.extra.Region", ""),
        "--type-name",
        _.get(component, "properties.domain.extra.AwsResourceType", ""),
        "--identifier",
        identifier,
      ]);

      if (child.exitCode !== 0) {
        console.log("Failed to refresh cloud control resource");
        console.log(child.stdout);
        console.error(child.stderr);
        return {
          status: "error",
          payload: _.get(component, "properties.resource.payload"),
          message:
            \`Refresh error; exit code \${child.exitCode}.\n\nSTDOUT:\n\n\${child.stdout}\n\nSTDERR:\n\n\${child.stderr}\`,
        };
      }

      const resourceResponse = JSON.parse(child.stdout);
      const payload = JSON.parse(
        resourceResponse["ResourceDescription"]["Properties"],
      );
      return {
        payload,
        status: "ok",
      };
    } else {
      return {
        message,
        payload: _.get(component, "properties.resource.payload"),
        status: "error",
      };
    }
  }

  // If you change this, you should change the same func on awsCloudControlCreate.ts in this same directory
  function addSecretsToPayload(
    payload: Record<string, any>,
    propUsageMap: {
      secrets: {
        secretKey: string;
        propPath: string[];
      }[];
    },
  ) {
    if (
      !Array.isArray(propUsageMap.secrets)
    ) {
      throw Error("malformed propUsageMap on asset");
    }

    for (
      const {
        secretKey,
        propPath,
      } of propUsageMap.secrets
    ) {
      const secret = requestStorage.getItem(secretKey);

      if (!propPath?.length || propPath.length < 1) {
        throw Error("malformed secret on propUsageMap: bad propPath");
      }
      if (!secret) continue;

      let secretParent = payload;
      let propKey = propPath[0];
      for (let i = 1; i < propPath.length; i++) {
        const thisProp = secretParent[propKey];

        if (!thisProp) {
          break;
        }

        secretParent = secretParent[propKey];
        propKey = propPath[i];
      }

      // Only add secret to payload if the codegen output has it
      if (propKey in secretParent) {
        secretParent[propKey] = secret;
      }
    }
  }
  </action-update-example>`,
  `<action-manual-example>
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
        message: \`Unable to update the EKS Cluster Access Config, AWS CLI 2 exited with non zero code: \${updateResp.exitCode}\`,
      };
    }

    return {
      payload: resource,
      status: "ok",
    };
  }
  </action-manual-example>`
];

const funcCreateOrEditInputSchemaRaw = {
  changeSetId: z
    .string()
    .describe(
      "The change set to create or update a function in; functions cannot be created on HEAD",
    ),
  schemaId: z.string().optional().describe("The schema id the function is for. Required for creating a new function, not needed for updating one."),
  funcId: z.string().optional().describe("The id of the function to edit. If none is given, create a new function"),
  name: z.string().min(1).optional().describe("The name of the function. Required for creating a new function."),
  description: z.string().optional().describe("A description for the function"),
  functionType: z.enum(["qualification", "codegen", "management", "action"]).optional().describe("The type of the function. Required for creating a new function."),
  functionCode: z
    .string()
    .optional()
    .describe(functionCodeDescribe.join(" ")),
  actionKind: z.enum(["Create", "Destroy", "Refresh", "Update", "Manual"]).optional().describe("The kind of action function. Only required for new functions of the action type."),
};

const funcCreateOrEditOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z.object({
    funcId: z.string().describe("the function id"),
  }),
};
const funcCreateOrEditOutputSchema = z.object(funcCreateOrEditOutputSchemaRaw);
type FuncCreateOrEditOutputData = z.infer<typeof funcCreateOrEditOutputSchema>["data"];

export function funcCreateOrEditTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "funcCreateOrEdit",
        funcCreateOrEditOutputSchema,
      ),
      inputSchema: funcCreateOrEditInputSchemaRaw,
      outputSchema: funcCreateOrEditOutputSchemaRaw,
    },
    async ({ changeSetId, schemaId, funcId, functionCode, functionType, actionKind, ...requestBody }) => {
      return await withAnalytics(name, async () => {
        const validationIssues = validateFunctionCode(functionType, functionCode, actionKind);
                if (validationIssues.length > 0) {
                  return errorResponse({
                    message: "Function code failed validation. Upsert aborted.",
                    hints: validationIssues.map(i => `• ${i.message}`).join("\n"),
                  });
                }
        const siSchemasApi = new SchemasApi(apiConfig);
        const siFuncsApi = new FuncsApi(apiConfig);

        let touchedFuncId;
        try {
          if (funcId) {
            // update an existing function

            // first fetch existing data about the function
            const responseGetFunc = await siFuncsApi.getFunc({
              workspaceId: WORKSPACE_ID,
              changeSetId,
              funcId,
            });

            // fill the update request body with our new data or existing data if it didn't change
            const updateFuncV1Request = {
              code: functionCode ?? responseGetFunc.data.code,
              description: requestBody.description ?? responseGetFunc.data.description,
              displayName: requestBody.name ?? responseGetFunc.data.displayName,
            }

            // finally hit the luminork API endpoint with the update
            await siFuncsApi.updateFunc({
              workspaceId: WORKSPACE_ID,
              changeSetId,
              funcId,
              updateFuncV1Request,
            });

            touchedFuncId = funcId;
          } else {
            // create a new function

            // first run checks to make sure we have the required data
            if (!schemaId) {
              return errorResponse({
                message: "You cannot create a function without specifying the schemaId of the schema you want the function created on."
              });
            } else if (!requestBody.name) {
              return errorResponse({
                message: "You cannot create a function without giving it a name."
              });
            } else if (!functionType) {
              return errorResponse({
                message: "You cannot create a function without specifying which type of function you want to create. This tool can create qualification, codegen, management, and action functions."
              });
            }

            // then unlock the schema we will be creating a function on
            const responseUnlockSchema = await siSchemasApi.unlockSchema({
              workspaceId: WORKSPACE_ID,
              changeSetId,
              schemaId,
            });

            // prepare the function create parameters
            const schemaVariantId = responseUnlockSchema.data.unlockedVariantId;
            const createFuncParams = {
              workspaceId: WORKSPACE_ID,
              changeSetId,
              schemaId,
              schemaVariantId,
            };

            // use the correct funciton create endpoint based on the type of function
            if (functionType === "qualification") {
              const responseCreate = await siSchemasApi.createVariantQualification({
                ...createFuncParams,
                createVariantQualificationFuncV1Request: {
                  ...requestBody,
                  code: functionCode ?? DEFAULT_QUALIFICATION_FUNCTION,
                },
              });

              touchedFuncId = responseCreate.data.funcId;
            } else if (functionType === "codegen") {
              const responseCreate = await siSchemasApi.createVariantCodegen({
                ...createFuncParams,
                createVariantCodegenFuncV1Request: {
                  ...requestBody,
                  code: functionCode ?? DEFAULT_CODEGEN_FUNCTION,
                },
              });

              touchedFuncId = responseCreate.data.funcId;
            } else if (functionType === "management") {
              const responseCreate = await siSchemasApi.createVariantManagement({
                ...createFuncParams,
                createVariantManagementFuncV1Request: {
                  ...requestBody,
                  code: functionCode ?? DEFAULT_MANAGEMENT_FUNCTION,
                },
              });

              touchedFuncId = responseCreate.data.funcId;
            } else if (functionType === "action") {
              if (!actionKind) {
                return errorResponse({
                  message: "Action kind is required for action functions."
                });
              }
              // else if (actionKind !== "Manual") {
              //   TODO: Aaron - preemptively protect the user against duplicated action functions
              //   Currently if the user attempts to make a duplicated action function, there will be an error.
              //   To catch this error before it happens, we would need to do the following -
              //   - Get all the variantFuncIds for the unlocked schema variant
              //   - Get the function details for each of those funcIds
              //   - Check if any of those functions are action functions of the same kind as the one being created
              // }
              const responseCreate = await siSchemasApi.createVariantAction({
                ...createFuncParams,
                createVariantActionFuncV1Request: {
                  ...requestBody,
                  code: functionCode ?? DEFAULT_ACTION_FUNCTION,
                  kind: actionKind,
                },
              });

              touchedFuncId = responseCreate.data.funcId;
            } else {
              return errorResponse({
                message: "Currently the agent can only create qualification, codegen, management, and action functions."
              });
            }
          }

          const data: FuncCreateOrEditOutputData = {
            funcId: touchedFuncId,
          };
          return successResponse(data);
        } catch (error) {
          const anyError = error as any;
          if (anyError?.response?.data && JSON.stringify(anyError.response.data).includes("action with kind")) {
            return errorResponse({
              message: "An action of the same kind already exists and only one action of each kind is allowed, except for Manual.",
              hints: "Tell the user that they can't make more than one of this kind of action and ask if they want to make an action of a different kind."
            });
          }
          return errorResponse(error);
        }
      });
    },
  );
}
