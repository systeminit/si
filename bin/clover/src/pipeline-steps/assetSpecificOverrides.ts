import _ from "npm:lodash";
import _logger from "../logger.ts";
import { propPathToString } from "../spec/sockets.ts";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";
import {
  addPropSuggestSource,
  createScalarProp,
  ExpandedPropSpec,
  ExpandedPropSpecFor,
  findPropByName,
} from "../spec/props.ts";
import { PropUsageMap } from "./addDefaultPropsAndSockets.ts";
import {
  ACTION_FUNC_SPECS,
  createActionFuncSpec,
  createFunc,
  createLeafFuncSpec,
  MANAGEMENT_FUNCS,
  modifyFunc,
  strippedBase64,
} from "../spec/funcs.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { FuncArgumentSpec } from "../bindings/FuncArgumentSpec.ts";
import { ActionFuncSpecKind } from "../bindings/ActionFuncSpecKind.ts";
import { FuncSpec } from "../bindings/FuncSpec.ts";
import { ActionFuncSpec } from "../bindings/ActionFuncSpec.ts";
import { LeafFunctionSpec } from "../bindings/LeafFunctionSpec.ts";

const logger = _logger.ns("assetOverrides").seal();

export function assetSpecificOverrides(
  incomingSpecs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const newSpecs = [] as ExpandedPkgSpec[];

  for (const spec of incomingSpecs) {
    if (overrides.has(spec.name)) {
      logger.debug(`Running override for ${spec.name}`);
      overrides.get(spec.name)?.(spec);
    }
    newSpecs.push(spec);
  }

  return newSpecs;
}

type OverrideFn = (spec: ExpandedPkgSpec) => void;

const overrides = new Map<string, OverrideFn>([
  // TODO prop suggestion ValueFrom <- Id
  // ["ContainerDefinitions Secrets", (spec: ExpandedPkgSpec) => {
  //   const variant = spec.schemas[0].variants[0];

  //   const prop = propForOverride(variant.domain, "ValueFrom");
  //   if (!prop) return;
  //   const socket = createInputSocketFromProp(prop);
  //   setAnnotationOnSocket(socket, { tokens: ["Id"] });
  //   variant.sockets.push(socket);
  // }],

  [
    "AWS::EC2::Instance",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];

      const overrideUserDataAttributeFuncCode = Deno.readTextFileSync(
        "./src/cloud-control-funcs/overrides/AWS::EC2::Instance/attribute/base64EncodeUserData.ts",
      );
      const overrideUserDataAttributeFuncArgs: FuncArgumentSpec[] = [
        {
          name: "data",
          kind: "string",
          elementKind: null,
          uniqueId: ulid(),
          deleted: false,
        },
      ];

      const base64EncodedUserDataFunc = createFunc(
        "Set UserData prop and base64 encode if needed",
        "jsAttribute",
        "json",
        strippedBase64(overrideUserDataAttributeFuncCode),
        "5a5b8c9d6e7f8a5b0c1d2e3f4a5b6c7d8e9f0a1b2c3d8e5f6a7b8c9d0e1f2a3b",
        overrideUserDataAttributeFuncArgs,
      );
      spec.funcs.push(base64EncodedUserDataFunc);

      const userDataProp = propForOverride(variant.domain, "UserData");
      userDataProp.data.widgetKind = "CodeEditor";

      // Create the Reboot Action
      const { func: rebootFunc, actionFuncSpec: rebootActionFuncSpec } =
        attachExtraActionFunction(
          "./src/cloud-control-funcs/overrides/AWS::EC2::Instance/actions/reboot.ts",
          "Reboot Ec2 Instance",
          "other",
          "5e38470604abb5c3ccc2ab60b31c5c0a05e9b381a2db73a15f4f8d55ec441bbd",
        );
      spec.funcs.push(rebootFunc);
      variant.actionFuncs.push(rebootActionFuncSpec);

      // Create the Stop Action
      const { func: stopFunc, actionFuncSpec: stopActionFuncSpec } =
        attachExtraActionFunction(
          "./src/cloud-control-funcs/overrides/AWS::EC2::Instance/actions/stop.ts",
          "Stop Ec2 Instance",
          "other",
          "de2c03b1caff5e7a1011a8c0ac6dc6dc99af77d15d0bc1f93e7c4eb9d7307f22",
        );
      spec.funcs.push(stopFunc);
      variant.actionFuncs.push(stopActionFuncSpec);

      // Create the Start Action
      const { func: startFunc, actionFuncSpec: startActionFuncSpec } =
        attachExtraActionFunction(
          "./src/cloud-control-funcs/overrides/AWS::EC2::Instance/actions/start.ts",
          "Start Ec2 Instance",
          "other",
          "f78a129cebfdb45c688df8622056e5ee2b81a41d8896c2ce7b24d0a709102d1f",
        );
      spec.funcs.push(startFunc);
      variant.actionFuncs.push(startActionFuncSpec);
    },
  ],
  [
    "AWS::EC2::LaunchTemplate",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];

      // TODO prop suggestions
      // ensure we can connect to the Version input for the EC2 Instance and AutoScaling Group Assets
      // const defaultVersionSocket = variant.sockets.find(
      //   (s: ExpandedSocketSpec) =>
      //     s.name === "Default Version Number" && s.data.kind === "output",
      // );
      // if (!defaultVersionSocket) return;
      // setAnnotationOnSocket(defaultVersionSocket, { tokens: ["Version"] });

      // const latestVersionSocket = variant.sockets.find(
      //   (s: ExpandedSocketSpec) =>
      //     s.name === "Latest Version Number" && s.data.kind === "output",
      // );
      // if (!latestVersionSocket) return;
      // setAnnotationOnSocket(latestVersionSocket, { tokens: ["Version"] });

      const ltData = objectPropForOverride(
        variant.domain,
        "LaunchTemplateData",
      );

      const prop = propForOverride(ltData, "UserData");
      prop.data.widgetKind = "CodeEditor";

      const importTargetId = MANAGEMENT_FUNCS["Import from AWS"].id;
      const newImportId =
        "0583c411a5b41594706ae8af473ed6d881357a1e692fb53981417f625f99374b";
      const importPath =
        "./src/cloud-control-funcs/overrides/AWS::EC2::LaunchTemplate/import.ts";

      modifyFunc(spec, importTargetId, newImportId, importPath);

      const discoverTargetId = MANAGEMENT_FUNCS["Discover on AWS"].id;
      const newDiscoverId =
        "cfebba8fc2d7cd88e5fc2b0c47a777b3737b8c2bcb88fbbb143be48018f22836";
      const discoverPath =
        "./src/cloud-control-funcs/overrides/AWS::EC2::LaunchTemplate/discover.ts";

      modifyFunc(spec, discoverTargetId, newDiscoverId, discoverPath);
    },
  ],
  // TODO prop suggestion
  // [
  //   "AWS::EC2::NetworkInterface",
  //   (spec: ExpandedPkgSpec) => {
  //     const variant = spec.schemas[0].variants[0];
  //     // Add an annotation for the Id output socket to connect to HostedZoneId
  //     const socket = variant.sockets.find(
  //       (s: ExpandedSocketSpec) => s.name === "Id" && s.data.kind === "output",
  //     );
  //     if (!socket) return;
  //     setAnnotationOnSocket(socket, { tokens: ["NetworkInterfaceId"] });
  //     const prop = propForOverride(variant.domain, "GroupSet");
  //     const groupSocket = createInputSocketFromProp(prop);
  //     setAnnotationOnSocket(groupSocket, { tokens: ["GroupId"] });
  //     variant.sockets.push(groupSocket);
  //   },
  // ],
  [
    "AWS::EC2::Route",
    (_spec: ExpandedPkgSpec) => {
      // TODO prop suggestions for GatewayId <- InternetGatewayId/VPNGatewayId
      // const variant = spec.schemas[0].variants[0];
      // const prop = propForOverride(variant.domain, "GatewayId");
      // const socket = createInputSocketFromProp(prop);
      // setAnnotationOnSocket(socket, { tokens: ["InternetGatewayId"] });
      // setAnnotationOnSocket(socket, { tokens: ["VPNGatewayId"] });
      // setAnnotationOnSocket(socket, { tokens: ["Gateway Id"] });
      // TODO prop suggestion for EgressOnlyInternetGatewayId <- Id
      // const egressOnlyIgwProp = propForOverride(
      //   variant.domain,
      //   "EgressOnlyInternetGatewayId",
      // );
      // const egressOnlyIgwInputSocket =
      //   createInputSocketFromProp(egressOnlyIgwProp);
      // setAnnotationOnSocket(egressOnlyIgwInputSocket, { tokens: ["Id"] });
      // setAnnotationOnSocket(egressOnlyIgwInputSocket, {
      //   tokens: ["Id<string<scalar>>"],
      // });
    },
  ],
  [
    "AWS::EC2::VPCEndpoint",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];

      const prop = propForOverride(variant.domain, "PolicyDocument");
      prop.kind = "json";
      prop!.data.widgetKind = "CodeEditor";
    },
  ],
  [
    "AWS::KMS::Key",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];

      const prop = propForOverride(variant.domain, "KeyPolicy");
      prop.kind = "json";
      prop!.data.widgetKind = "CodeEditor";
    },
  ],
  [
    "AWS::Logs::LogGroup",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];

      const prop = propForOverride(variant.domain, "DataProtectionPolicy");
      prop.kind = "json";
      prop!.data.widgetKind = "CodeEditor";
    },
  ],
  [
    "AWS::RDS::DBParameterGroup",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];

      const prop = propForOverride(variant.domain, "Parameters");
      prop.kind = "map";

      if (prop.kind === "map") {
        prop.typeProp = createScalarProp(
          "parameter",
          "string",
          prop.metadata.propPath,
          false,
        );
      }
    },
  ],
  [
    "AWS::SecretsManager::Secret",
    (spec: ExpandedPkgSpec) => {
      addSecretProp("Secret String", "secretString", ["SecretString"])(spec);
    },
  ],
  [
    "AWS::RDS::DBCluster",
    addSecretProp("Secret String", "secretString", ["MasterUserPassword"]),
  ],
  [
    "AWS::RDS::DBInstance",
    (spec: ExpandedPkgSpec) => {
      addSecretProp("Secret String", "secretString", ["MasterUserPassword"])(
        spec,
      );

      // TODO prop suggestions
      // const variant = spec.schemas[0].variants[0];

      // const securityGroupsSocket = variant.sockets.find(
      //   (s: ExpandedSocketSpec) =>
      //     s.name === "VPC Security Groups" && s.data.kind === "input",
      // );
      // if (!securityGroupsSocket) return;

      // setAnnotationOnSocket(securityGroupsSocket, { tokens: ["GroupId"] });
    },
  ],
  // TODO duplicate with other prop suggestion override
  // [
  //   "AWS::EC2::NetworkInterface",
  //   (spec: ExpandedPkgSpec) => {
  //     const variant = spec.schemas[0].variants[0];

  //     const prop = propForOverride(variant.domain, "GroupSet");
  //     const groupSocket = createInputSocketFromProp(prop);

  //     setAnnotationOnSocket(groupSocket, { tokens: ["GroupId"] });
  //   },
  // ],
  [
    "AWS::EC2::SecurityGroupIngress",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];
      const domainId = variant.domain.uniqueId;

      if (!domainId) return;

      // TODO prop suggestions
      // // Add Source SG ID to an input socket
      // const idProp = propForOverride(variant.domain, "SourceSecurityGroupId");
      // const groupIdSocket = createInputSocketFromProp(idProp);

      // setAnnotationOnSocket(groupIdSocket, {
      //   tokens: ["SourceSecurityGroupId", "GroupId"],
      // });
      // setAnnotationOnSocket(groupIdSocket, { tokens: ["GroupId"] });
      // variant.sockets.push(groupIdSocket);

      // // Add Source SG Name to an input socket
      // const nameProp = propForOverride(
      //   variant.domain,
      //   "SourceSecurityGroupName",
      // );
      // const groupSocket = createInputSocketFromProp(nameProp);

      // setAnnotationOnSocket(groupSocket, {
      //   tokens: ["SourceSecurityGroupName", "GroupName"],
      // });
      // variant.sockets.push(groupSocket);

      const { func, leafFuncSpec } = attachQualificationFunction(
        "./src/cloud-control-funcs/overrides/AWS::EC2::SecurityGroupIngress/qualifications/checkForEitherGroupIdOrGroupName.ts",
        "GroupId OR GroupName",
        "23f026310223509f053b55bfa386772eecc2d00e3090dbeb65766ac63f8c53a2",
        domainId,
      );

      spec.funcs.push(func);
      variant.leafFunctions.push(leafFuncSpec);
    },
  ],
  // TODO prop suggestions
  // [
  //   "AWS::EC2::SecurityGroup",
  //   (spec: ExpandedPkgSpec) => {
  //     const variant = spec.schemas[0].variants[0];

  //     // Add SG GroupName to an output socket
  //     const nameProp = propForOverride(variant.domain, "GroupName");
  //     const groupSocket = createOutputSocketFromProp(nameProp);
  //     variant.sockets.push(groupSocket);

  //     // Add annotations to Group Id output socket
  //     const groupIdSocket = variant.sockets.find(
  //       (s: ExpandedSocketSpec) =>
  //         s.name === "Group Id" && s.data.kind === "output",
  //     );
  //     if (!groupIdSocket) return;

  //     setAnnotationOnSocket(groupIdSocket, { tokens: ["Security Group Ids"] });
  //     setAnnotationOnSocket(groupIdSocket, { tokens: ["Security Group Id"] });
  //     setAnnotationOnSocket(groupIdSocket, { tokens: ["GroupId"] });
  //   },
  // ],
  // TODO prop suggestions
  // [
  //   "AWS::EC2::SecurityGroupEgress",
  //   (spec: ExpandedPkgSpec) => {
  //     const variant = spec.schemas[0].variants[0];

  //     // Add Destination SG ID to an input socket
  //     const idProp = propForOverride(
  //       variant.domain,
  //       "DestinationSecurityGroupId",
  //     );
  //     const groupIdSocket = createInputSocketFromProp(idProp);

  //     setAnnotationOnSocket(groupIdSocket, {
  //       tokens: ["DestinationSecurityGroupId", "GroupId"],
  //     });
  //     setAnnotationOnSocket(groupIdSocket, { tokens: ["GroupId"] });
  //     variant.sockets.push(groupIdSocket);
  //   },
  // ],

  [
    "AWS::AutoScaling::AutoScalingGroup",
    (spec: ExpandedPkgSpec) => {
      // TODO prop suggestions
      const variant = spec.schemas[0].variants[0];
      // const launchTemplateVersionSocket = variant.sockets.find(
      //   (s: ExpandedSocketSpec) =>
      //     s.name === "Launch Template Version" && s.data.kind === "input",
      // );
      // if (!launchTemplateVersionSocket) return;

      // setAnnotationOnSocket(launchTemplateVersionSocket, {
      //   tokens: ["DefaultVersionNumber"],
      // });
      // setAnnotationOnSocket(launchTemplateVersionSocket, {
      //   tokens: ["LatestVersionNumber"],
      // });
      // setAnnotationOnSocket(launchTemplateVersionSocket, {
      //   tokens: ["LaunchTemplateVersion<string<scalar>>"],
      // });

      // Modify the existing update function instead of replacing it
      const updateTargetId = ACTION_FUNC_SPECS["Update Asset"].id;
      const newUpdateId =
        "c7e6bf82e9d7fa438f6a9151a1b1f4c6f4b18ae50eacf462bc81d2b31278e1c5";
      const updatePath =
        "./src/cloud-control-funcs/overrides/AWS::AutoScaling::AutoScalingGroup/actions/awsCloudControlUpdate.ts";
      modifyFunc(spec, updateTargetId, newUpdateId, updatePath);

      const {
        func: refreshInstancesFunc,
        actionFuncSpec: refreshInstancesFuncSpec,
      } = attachExtraActionFunction(
        "./src/cloud-control-funcs/overrides/AWS::AutoScaling::AutoScalingGroup/actions/instanceRefresh.ts",
        "Refresh Autoscaling Group Instances",
        "other",
        "300d62f40cb1268e6f4cd2320be8c373da7f148d0d9e6e69d1b2879202794b5f",
      );
      spec.funcs.push(refreshInstancesFunc);
      variant.actionFuncs.push(refreshInstancesFuncSpec);
    },
  ],
  // TODO prop suggestions
  // [
  //   "TargetGroup Targets",
  //   (spec: ExpandedPkgSpec) => {
  //     const variant = spec.schemas[0].variants[0];

  //     // Add an annotation for the Id output socket to connect to HostedZoneId
  //     const socket = variant.sockets.find(
  //       (s: ExpandedSocketSpec) => s.name === "Id" && s.data.kind === "input",
  //     );
  //     if (!socket) return;

  //     setAnnotationOnSocket(socket, { tokens: ["InstanceId"] });
  //     setAnnotationOnSocket(socket, { tokens: ["arn", "string"] });
  //     setAnnotationOnSocket(socket, { tokens: ["arn"] });
  //   },
  // ],
  [
    "AWS::ImageBuilder::Component",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];

      const prop = propForOverride(variant.domain, "Data");
      prop!.data.widgetKind = "CodeEditor";
    },
  ],
  [
    "AWS::S3::BucketPolicy",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];

      const prop = propForOverride(variant.domain, "PolicyDocument");
      prop.kind = "json";
      prop!.data.widgetKind = "CodeEditor";
    },
  ],
  // TODO prop suggestions
  // [
  //   "AWS::EC2::VPCCidrBlock",
  //   (spec: ExpandedPkgSpec) => {
  //     const variant = spec.schemas[0].variants[0];

  //     const ipv6IpamProp = propForOverride(variant.domain, "Ipv6IpamPoolId");

  //     const ipv6IpamSocket = createInputSocketFromProp(ipv6IpamProp, [
  //       { tokens: ["Ipam Pool Id"] },
  //       { tokens: ["IpamPoolId"] },
  //       { tokens: ["IpamPoolId", "string", "scalar"] },
  //     ]);
  //     const ipv4IpamProp = propForOverride(variant.domain, "Ipv4IpamPoolId");

  //     const ipv4IpamSocket = createInputSocketFromProp(ipv4IpamProp, [
  //       { tokens: ["Ipam Pool Id"] },
  //       { tokens: ["IpamPoolId"] },
  //       { tokens: ["IpamPoolId", "string", "scalar"] },
  //     ]);
  //   },
  // ],
  [
    "AWS::ECS::Cluster",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];
      const configurationProp = objectPropForOverride(
        variant.domain,
        "Configuration",
      );

      const managedStorageConfigurationProp = objectPropForOverride(
        configurationProp,
        "ManagedStorageConfiguration",
      );

      const fargateKmsProp = propForOverride(
        managedStorageConfigurationProp,
        "FargateEphemeralStorageKmsKeyId",
      );

      addPropSuggestSource(fargateKmsProp, {
        schema: "AWS:KMS:Key",
        prop: "/resource_value/KeyId",
      });

      const kmsKeyIdProp = propForOverride(
        managedStorageConfigurationProp,
        "KmsKeyId",
      );

      addPropSuggestSource(kmsKeyIdProp, {
        schema: "AWS:KMS:Key",
        prop: "/resource_value/KeyId",
      });
    },
  ],
  // TODO prop suggestions
  // [
  //   "AWS::EC2::VPCPeeringConnection",
  //   (spec: ExpandedPkgSpec) => {
  //     const variant = spec.schemas[0].variants[0];

  //     const peerVpcIdProp = propForOverride(variant.domain, "PeerVpcId");

  //     const peerVpcIdSocket = createInputSocketFromProp(
  //       peerVpcIdProp,
  //       [
  //         { tokens: ["VPC Id"] },
  //         { tokens: ["VpcId"] },
  //         { tokens: ["VpcId", "string", "scalar"] },
  //       ],
  //       "Peer Vpc Id",
  //     );

  //     const peerOwnerIdProp = propForOverride(variant.domain, "PeerOwnerId");

  //     const peerOwnerIdSocket = createInputSocketFromProp(
  //       peerOwnerIdProp,
  //       [
  //         { tokens: ["Account Id"] },
  //         { tokens: ["AccountId"] },
  //         { tokens: ["AccountId", "string", "scalar"] },
  //       ],
  //       "Peer Owner Id",
  //     );
  //   },
  // ],
  [
    "AWS::ApiGatewayV2::Route",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];

      const requestModelsProp = propForOverride(
        variant.domain,
        "RequestModels",
      );
      requestModelsProp.kind = "json";
      requestModelsProp!.data.widgetKind = "CodeEditor";

      const requestParametersProp = propForOverride(
        variant.domain,
        "RequestParameters",
      );
      requestParametersProp.kind = "json";
      requestParametersProp!.data.widgetKind = "CodeEditor";
    },
  ],
  // TODO prop suggestions HostedZoneId -> Id
  // [
  //   "Certificate DomainValidationOptions",
  //   (spec: ExpandedPkgSpec) => {
  //     const variant = spec.schemas[0].variants[0];

  //     const socket = variant.sockets.find(
  //       (s: ExpandedSocketSpec) =>
  //         s.name === "Hosted Zone Id" && s.data.kind === "input",
  //     );
  //     if (!socket) return;

  //     setAnnotationOnSocket(socket, { tokens: ["Id"] });
  //   },
  // ],
  // TODO prop suggestions SecurityGroups -> GroupId
  // [
  //   "AWS::ElasticLoadBalancingV2::LoadBalancer",
  //   (spec: ExpandedPkgSpec) => {
  //     const variant = spec.schemas[0].variants[0];

  //     // Add annotations to Security Groups input socket
  //     const securityGroupsSocket = variant.sockets.find(
  //       (s: ExpandedSocketSpec) =>
  //         s.name === "Security Groups" && s.data.kind === "input",
  //     );
  //     if (!securityGroupsSocket) return;

  //     setAnnotationOnSocket(securityGroupsSocket, { tokens: ["GroupId"] });
  //   },
  // ],
  [
    "Listener DefaultActions",
    (spec: ExpandedPkgSpec) => {
      const targetGroupArnToListAttributeFunc = Deno.readTextFileSync(
        "./src/cloud-control-funcs/overrides/Listener DefaultActions/attribute/setTargetGroupArns.ts",
      );
      const targetGroupArnToListFuncArgs: FuncArgumentSpec[] = [
        {
          name: "targetGroupArn",
          kind: "string",
          elementKind: null,
          uniqueId: ulid(),
          deleted: false,
        },
        {
          name: "type",
          kind: "string",
          elementKind: null,
          uniqueId: ulid(),
          deleted: false,
        },
      ];

      const attrFuncId =
        "2930eb8752c80c973163b3dec9cc2465b98d6257a49469c823429da4031a1c32";
      const targetGroupArnToListFunc = createFunc(
        "Set Forward Config if Target Group Arn is set",
        "jsAttribute",
        "json",
        strippedBase64(targetGroupArnToListAttributeFunc),
        attrFuncId,
        targetGroupArnToListFuncArgs,
      );
      spec.funcs.push(targetGroupArnToListFunc);

      const variant = spec.schemas[0].variants[0];
      const forwardConfigProp = objectPropForOverride(
        variant.domain,
        "ForwardConfig",
      );

      const targetGroupsProp = propForOverride(
        forwardConfigProp,
        "TargetGroups",
      );
      targetGroupsProp.data.funcUniqueId = attrFuncId;
      targetGroupsProp.data.inputs = [
        {
          kind: "prop",
          name: "type",
          prop_path: propPathToString(["root", "domain", "Type"]),
          unique_id: ulid(),
          deleted: false,
        },
        {
          kind: "prop",
          name: "targetGroupArn",
          prop_path: propPathToString(["root", "domain", "TargetGroupArn"]),
          unique_id: ulid(),
          deleted: false,
        },
      ];
    },
  ],
  // TODO prop suggestions Image -> RepositoryUri
  // [
  //   "TaskDefinition ContainerDefinitions",
  //   (spec: ExpandedPkgSpec) => {
  //     const variant = spec.schemas[0].variants[0];

  //     const imageSocket = variant.sockets.find(
  //       (s: ExpandedSocketSpec) =>
  //         s.name === "Image" && s.data.kind === "input",
  //     );
  //     if (!imageSocket) return;

  //     setAnnotationOnSocket(imageSocket, { tokens: ["repositoryuri"] });
  //     setAnnotationOnSocket(imageSocket, { tokens: ["repository uri"] });
  //   },
  // ],

  // TODO prop suggestions Role -> Arn
  // [
  //   "AWS::Lambda::Function",
  //   (spec: ExpandedPkgSpec) => {
  //     const variant = spec.schemas[0].variants[0];

  //     const roleSocket = variant.sockets.find(
  //       (s: ExpandedSocketSpec) => s.name === "Role" && s.data.kind === "input",
  //     );
  //     if (!roleSocket) return;

  //     setAnnotationOnSocket(roleSocket, { tokens: ["arn", "string"] });
  //     setAnnotationOnSocket(roleSocket, { tokens: ["arn"] });
  //   },
  // ],
  [
    "AWS::ECS::TaskDefinition",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];

      // List of props to remove read-only
      const propsToRemove = [
        "ContainerDefinitions",
        "Cpu",
        "EnableFaultInjection",
        "ExecutionRoleArn",
        "InferenceAccelerators",
        "Memory",
        "NetworkMode",
        "PlacementConstraints",
        "ProxyConfiguration",
        "RequiresCompatibilities",
        "RuntimePlatform",
        "TaskRoleArn",
        "Volumes",
        "PidMode",
        "IpcMode",
        "EphemeralStorage",
      ];

      const containerDefsProp = arrayPropForOverride(
        variant.domain,
        "ContainerDefinitions",
      );
      let itemProp = containerDefsProp.typeProp;
      itemProp = addPropSuggestSource(itemProp, {
        schema: "TaskDefinition ContainerDefinitions",
        prop: "/domain",
      });

      const extraProp = objectPropForOverride(variant.domain, "extra");

      const propUsageMapProp = propForOverride(extraProp, "PropUsageMap");
      if (!propUsageMapProp.data?.defaultValue)
        throw new Error("Prop has no default value");

      const defaultValue = JSON.parse(
        propUsageMapProp.data.defaultValue as string,
      );
      let createOnly = defaultValue["createOnly"];
      const updatable = defaultValue["updatable"];

      propsToRemove.forEach((propName) => {
        const prop = propForOverride(variant.domain, propName);

        const currentWidgetOptions = prop.data.widgetOptions;
        prop.data.widgetOptions =
          currentWidgetOptions?.filter(
            (w) => w.label !== "si_create_only_prop",
          ) ?? null;

        createOnly = createOnly?.filter((p: string) => p !== propName);

        updatable.push(propName);
      });

      defaultValue["createOnly"] = createOnly;
      defaultValue["updatable"] = updatable;
      propUsageMapProp!.data.defaultValue = JSON.stringify(defaultValue);

      const updateTargetId = ACTION_FUNC_SPECS["Update Asset"].id;
      const newUpdateId =
        "7eb4e58626f9fd7ee003bb9a1de814ab31cbb8ea2ae87d844864058bc4296c63";
      const newUpdatePath =
        "./src/cloud-control-funcs/overrides/AWS::ECS::TaskDefinition/actions/update.ts";
      modifyFunc(spec, updateTargetId, newUpdateId, newUpdatePath);
    },
  ],
  [
    "AWS::ECR::RegistryPolicy",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];

      const policyTextProp = propForOverride(variant.domain, "PolicyText");

      // PolicyText needs to be prop kind json and widgetkind CodeEditor
      policyTextProp.kind = "json";
      policyTextProp.data.widgetKind = "CodeEditor";

      // TODO prop suggestion PolicyText -> PolicyDocument
      // // Create an input socket that connects PolicyDocument
      // const policyDocumentSocket = createInputSocketFromProp(
      //   policyTextProp,
      //   [{ tokens: ["policydocument"] }],
      //   "Policy Document",
      // );
      // variant.sockets.push(policyDocumentSocket);
    },
  ],
  // TODO prop suggestion Name -> ClusterName
  // [
  //   "AWS::EKS::Cluster",
  //   (spec: ExpandedPkgSpec) => {
  //     const variant = spec.schemas[0].variants[0];

  //     const nameOutputSocket = variant.sockets.find(
  //       (s: ExpandedSocketSpec) =>
  //         s.name === "Name" && s.data.kind === "output",
  //     );
  //     if (!nameOutputSocket) return;

  //     setAnnotationOnSocket(nameOutputSocket, { tokens: ["Cluster Name"] });
  //     setAnnotationOnSocket(nameOutputSocket, { tokens: ["ClusterName"] });
  //   },
  // ],

  // TODO prop suggestion ResourceArn -> LoadBalancerArn
  // [
  //   "AWS::WAFv2::WebACLAssociation",
  //   (spec: ExpandedPkgSpec) => {
  //     const variant = spec.schemas[0].variants[0];

  //     const resourceArnSocket = variant.sockets.find(
  //       (s: ExpandedSocketSpec) =>
  //         s.name === "Resource Arn" && s.data.kind === "output",
  //     );
  //     if (!resourceArnSocket) return;
  //     setAnnotationOnSocket(resourceArnSocket, { tokens: ["LoadBalancerArn"] });
  //   },
  // ],

  [
    "AWS::EKS::Nodegroup",
    (_spec: ExpandedPkgSpec) => {
      // TODO prop suggestion RemoteAccess.Ec2SshKey <- KeyName/KeyPair/SshKey
      // const variant = spec.schemas[0].variants[0];
      // const domain = variant.domain;
      // const remoteAccessProp = objectPropForOverride(domain, "RemoteAccess");
      // const ec2SshKeyProp = propForOverride(remoteAccessProp, "Ec2SshKey");
      // const sshKeySocket = createInputSocketFromProp(
      //   ec2SshKeyProp,
      //   [
      //     { tokens: ["KeyName"] },
      //     { tokens: ["Key Name"] },
      //     { tokens: ["KeyPair"] },
      //     { tokens: ["ssh key"] },
      //   ],
      //   "Key Name",
      // );
      // TODO prop suggestion NodeRole <- RoleArn/Role ARN/IAM Role/Arn/Role
      // const nodeRoleProp = propForOverride(domain, "NodeRole");
      // const nodeRoleSocket = createInputSocketFromProp(
      //   nodeRoleProp,
      //   [
      //     { tokens: ["RoleArn"] },
      //     { tokens: ["Role ARN"] },
      //     { tokens: ["IAM Role"] },
      //     { tokens: ["arn"] },
      //     { tokens: ["role"] },
      //   ],
      //   "Node Role",
      // );
    },
  ],
  // TODO prop suggestion SubnetIds -> SubnetId
  // [
  //   "AWS::RDS::DBSubnetGroup",
  //   (spec: ExpandedPkgSpec) => {
  //     const variant = spec.schemas[0].variants[0];

  //     const subnetIdSocket = variant.sockets.find(
  //       (s: ExpandedSocketSpec) =>
  //         s.name === "Subnet Ids" && s.data.kind === "input",
  //     );
  //     if (!subnetIdSocket) return;
  //     setAnnotationOnSocket(subnetIdSocket, { tokens: ["subnet id"] });
  //   },
  // ],
  [
    "VPNConnection VpnTunnelOptionsSpecifications",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];
      const domainId = variant.domain.uniqueId;

      if (!domainId) return;

      const { func, leafFuncSpec } = attachQualificationFunction(
        "./src/cloud-control-funcs/overrides/VPNConnection VpnTunnelOptionsSpecifications/qualifications/presharedKeyValidations.ts",
        "Validate PresharedKey",
        "4e1243bd22c67dd61b0a4c86e9f3c89c84baf7b37a45f93ee4e5ed8a9d7f1c2f",
        domainId,
      );

      spec.funcs.push(func);
      variant.leafFunctions.push(leafFuncSpec);

      const propsToFix = [
        "IKEVersions",
        "Phase1DHGroupNumbers",
        "Phase1EncryptionAlgorithms",
        "Phase1IntegrityAlgorithms",
        "Phase2DHGroupNumbers",
        "Phase2EncryptionAlgorithms",
        "Phase2IntegrityAlgorithms",
      ];

      for (const propName of propsToFix) {
        const arrayProp = arrayPropForOverride(variant.domain, propName);

        arrayProp.data.widgetKind = "Array";
        arrayProp.data.inputs = [];
        arrayProp.data.funcUniqueId = null;

        if (arrayProp.typeProp.kind === "object") {
          const valueProp = arrayProp.typeProp.entries.find(
            (p) => p.name === "Value",
          );
          if (valueProp) {
            valueProp.data.widgetKind = "ComboBox";
          }
        }
      }
    },
  ],

  // TODO prop suggestion SubnetIds <- SubnetId/Subnets
  // [
  //   "AWS::ElastiCache::ServerlessCache",
  //   (spec: ExpandedPkgSpec) => {
  //     const variant = spec.schemas[0].variants[0];
  //     const subnetIdSocket = variant.sockets.find(
  //       (s: ExpandedSocketSpec) =>
  //         s.name === "Subnet Ids" && s.data.kind === "input",
  //     );
  //     if (!subnetIdSocket) return;
  //     setAnnotationOnSocket(subnetIdSocket, { tokens: ["subnet id"] });
  //     setAnnotationOnSocket(subnetIdSocket, { tokens: ["subnetid"] });
  //     setAnnotationOnSocket(subnetIdSocket, { tokens: ["subnets"] });
  //   },
  // ],

  // TODO prop suggestion SubnetIds <- SubnetId/Subnets
  // [
  //   "AWS::ElastiCache::SubnetGroup",
  //   (spec: ExpandedPkgSpec) => {
  //     const variant = spec.schemas[0].variants[0];
  //     const subnetIdSocket = variant.sockets.find(
  //       (s: ExpandedSocketSpec) =>
  //         s.name === "Subnet Ids" && s.data.kind === "input",
  //     );
  //     if (!subnetIdSocket) return;
  //     setAnnotationOnSocket(subnetIdSocket, { tokens: ["subnet id"] });
  //     setAnnotationOnSocket(subnetIdSocket, { tokens: ["subnetid"] });
  //     setAnnotationOnSocket(subnetIdSocket, { tokens: ["subnets"] });
  //   },
  // ],

  // TODO prop suggestion Id -> TransitGatewayId
  // [
  //   "AWS::EC2::TransitGateway",
  //   (spec: ExpandedPkgSpec) => {
  //     const variant = spec.schemas[0].variants[0];

  //     const idSocket = variant.sockets.find(
  //       (s: ExpandedSocketSpec) => s.name === "Id" && s.data.kind === "output",
  //     );
  //     if (!idSocket) return;
  //     setAnnotationOnSocket(idSocket, { tokens: ["TransitGatewayId"] });
  //     setAnnotationOnSocket(idSocket, { tokens: ["TransitGatewayId"] });
  //     setAnnotationOnSocket(idSocket, { tokens: ["Transit Gateway Id"] });
  //   },
  // ],

  [
    "AWS::EC2::VPNConnection",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];
      const resource_value = variant.resourceValue;

      const typeProp = stringPropForOverride(variant.domain, "Type");
      typeProp.data.widgetKind = "ComboBox";
      typeProp.data.defaultValue = "ipsec.1";
      typeProp.data.inputs = [];
      typeProp.data.funcUniqueId = null;
      typeProp.data.widgetOptions = [
        { label: "si_create_only_prop", value: "true" },
      ];

      const transitGatewayAttachmentIdProp = createScalarProp(
        "TransitGatewayAttachmentId",
        "string",
        ["root", "resource_value"],
        false,
      );
      transitGatewayAttachmentIdProp.data.widgetKind = "Text";
      resource_value.entries.push(transitGatewayAttachmentIdProp);

      const refreshTargetId = ACTION_FUNC_SPECS["Refresh Asset"].id;
      const newRefreshId =
        "fd3706e543528a703c674f42c07d3f2443b2e3c40bfc88a81a7f4501af5e7122";
      const refreshPath =
        "./src/cloud-control-funcs/overrides/AWS::EC2::VPNConnection/actions/refresh.ts";
      modifyFunc(spec, refreshTargetId, newRefreshId, refreshPath);
    },
  ],

  // TODO prop suggestion SubnetIds <- Subnets
  // [
  //   "AWS::EC2::TransitGatewayAttachment",
  //   (spec: ExpandedPkgSpec) => {
  //     const variant = spec.schemas[0].variants[0];
  //     const subnetInputSocket = variant.sockets.find(
  //       (s: ExpandedSocketSpec) =>
  //         s.name === "Subnet Ids" && s.data.kind === "input",
  //     );
  //     if (!subnetInputSocket) return;
  //     setAnnotationOnSocket(subnetInputSocket, { tokens: ["subnets"] });
  //   },
  // ],

  [
    "AWS::EC2::CustomerGateway",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];

      const typeProp = stringPropForOverride(variant.domain, "Type");
      typeProp.data.widgetKind = "ComboBox";
      typeProp.data.defaultValue = "ipsec.1";
      typeProp.data.inputs = [];
      typeProp.data.funcUniqueId = null;
      typeProp.data.widgetOptions = [
        { label: "si_create_only_prop", value: "true" },
      ];
    },
  ],
  [
    "AWS::S3::Bucket",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];
      const domain = variant.domain;

      const bucketNameProp = stringPropForOverride(domain, "BucketName");
      bucketNameProp.data.widgetKind = "Text";
      bucketNameProp.data.inputs = [];
      bucketNameProp.data.funcUniqueId = null;
      bucketNameProp.data.widgetOptions = [
        { label: "si_create_only_prop", value: "true" },
      ];
    },
  ],

  // TODO prop suggestion DistributionViewerCertificate.AcmCertificateArn <- CertificateArn
  // [
  //   "AWS::CloudFront::Distribution",
  //   (spec: ExpandedPkgSpec) => {
  //     const variant = spec.schemas[0].variants[0];

  //     const certificateInputSocket = variant.sockets.find(
  //       (s: ExpandedSocketSpec) =>
  //         s.name === "Distribution Viewer Certificate Acm Certificate Arn" &&
  //         s.data.kind === "input",
  //     );
  //     if (!certificateInputSocket) return;
  //     setAnnotationOnSocket(certificateInputSocket, {
  //       tokens: ["certificate arn"],
  //     });
  //   },
  // ],
]);

function attachExtraActionFunction(
  funcPath: string,
  name: string,
  kind: ActionFuncSpecKind,
  uniqueId: string,
): { func: FuncSpec; actionFuncSpec: ActionFuncSpec } {
  const funcCode = Deno.readTextFileSync(funcPath);
  const func = createFunc(
    name,
    "jsAction",
    "action",
    strippedBase64(funcCode),
    uniqueId,
    [],
  );
  func.data!.displayName = name;

  const actionFuncSpec = createActionFuncSpec(kind, func.uniqueId);

  return { func, actionFuncSpec };
}

function attachQualificationFunction(
  funcPath: string,
  name: string,
  uniqueId: string,
  domainId: string,
): { func: FuncSpec; leafFuncSpec: LeafFunctionSpec } {
  const funcCode = Deno.readTextFileSync(funcPath);

  const func = createFunc(
    name,
    "jsAttribute",
    "qualification",
    strippedBase64(funcCode),
    uniqueId,
    [
      {
        name: "domain",
        kind: "object",
        elementKind: null,
        uniqueId: domainId,
        deleted: false,
      },
    ],
  );
  func.data!.displayName = name;

  const leafFuncSpec = createLeafFuncSpec("qualification", func.uniqueId, [
    "domain",
  ]);

  return { func, leafFuncSpec };
}

function addSecretProp(
  secretKind: string,
  secretKey: string,
  propPath: string[],
) {
  return (spec: ExpandedPkgSpec) => {
    const variant = spec.schemas[0].variants[0];

    const [secretName] = propPath.slice(-1);
    if (!secretName) {
      return;
    }

    // Find secret prop
    let secretParent = variant.domain;
    let secretProp: ExpandedPropSpec | undefined = variant.domain;

    for (const propName of propPath) {
      // If we haven't found the secret prop yet, and we're not with an object in hand, break
      if (secretProp.kind !== "object") {
        secretProp = undefined;
        break;
      }

      secretParent = secretProp;
      const thisProp = secretParent.entries.find((p) => p.name === propName);

      // If we don't find the prop on the parent, break
      if (!thisProp) {
        secretProp = undefined;
        break;
      }

      secretProp = thisProp;
    }

    if (!secretProp) {
      console.log(`Could not add secret value for ${spec.name}`);
      return;
    }

    // Find propUsageMap
    const extraProp = variant.domain.entries.find((p) => p.name === "extra");
    if (extraProp?.kind !== "object") {
      return;
    }
    const propUsageMapProp = extraProp.entries.find(
      (p) => p.name === "PropUsageMap",
    );
    const propUsageMap = JSON.parse(
      propUsageMapProp?.data.defaultValue,
    ) as PropUsageMap;

    if (!propUsageMapProp || !Array.isArray(propUsageMap?.secrets)) {
      return;
    }

    // Remove secret from the domain tree
    secretParent.entries = secretParent.entries.filter(
      (p: ExpandedPropSpec) => p.name !== secretName,
    );

    // Add prop to secrets tree
    secretProp.data.widgetKind = "Secret";
    secretProp.data.widgetOptions = [
      {
        label: "secretKind",
        value: secretKind,
      },
    ];
    variant.secrets.entries.push(secretProp);
    // Replace "domain" with "secrets" on propPath
    secretProp.metadata.propPath[1] = "secrets";

    // TODO prop suggestion?
    // // Add socket for secret prop
    // const secretStringProp = createInputSocketFromProp(secretProp);
    // variant.sockets.push(secretStringProp);
    // setAnnotationOnSocket(secretStringProp, { tokens: [secretKind] });

    // add secret to the propUsageMap
    propUsageMap.secrets.push({
      secretKey,
      propPath,
    });
    propUsageMapProp.data.defaultValue = JSON.stringify(propUsageMap);
  };
}

function propForOverride(
  objPropSpec: ExpandedPropSpecFor["object"],
  propName: string,
): ExpandedPropSpec {
  const prop = findPropByName(objPropSpec, propName);
  if (!prop) {
    throw new Error(
      `Prop ${propName} not found under ${objPropSpec.name} for override!`,
    );
  }
  return prop;
}

function arrayPropForOverride(
  objPropSpec: ExpandedPropSpecFor["object"],
  propName: string,
): ExpandedPropSpecFor["array"] {
  const prop = propForOverride(objPropSpec, propName);
  if (prop?.kind !== "array") {
    throw new Error(`Prop ${propName} is not an array!`);
  }
  return prop;
}

function objectPropForOverride(
  objPropSpec: ExpandedPropSpecFor["object"],
  propName: string,
): ExpandedPropSpecFor["object"] {
  const prop = propForOverride(objPropSpec, propName);
  if (prop?.kind !== "object") {
    throw new Error(`Prop ${propName} is not an object!`);
  }
  return prop;
}

function stringPropForOverride(
  objPropSpec: ExpandedPropSpecFor["object"],
  propName: string,
): ExpandedPropSpecFor["string"] {
  const prop = propForOverride(objPropSpec, propName);
  if (prop?.kind !== "string") {
    throw new Error(`Prop ${propName} is not a string!`);
  }
  return prop;
}
