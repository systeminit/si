import { PropOverrideFn, SchemaOverrideFn } from "../types.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { ACTION_FUNC_SPECS, MANAGEMENT_FUNCS } from "./funcs.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { FuncArgumentSpec } from "../../bindings/FuncArgumentSpec.ts";
import {
  addSecretProp,
  arnProp,
  arrayPropForOverride,
  attachExtraActionFunction,
  attachQualificationFunction,
  objectPropForOverride,
  policyDocumentProp,
  propForOverride,
  stringPropForOverride,
  suggest,
} from "../generic/overrides.ts";
import {
  addPropSuggestSource,
  createScalarProp,
} from "../../spec/props.ts";
import { createFunc, modifyFunc, strippedBase64 } from "../../spec/funcs.ts";

// AWS-specific property overrides
export const AWS_PROP_OVERRIDES: Record<
  string,
  Record<string, PropOverrideFn | PropOverrideFn[]>
> = {
  // AWS::EC2
  "AWS::EC2::FlowLog": {
    DeliverLogsPermissionArn: arnProp("AWS::IAM::Role"),
  },
  "AWS::EC2::LaunchTemplate": {
    "LaunchTemplateData/LicenseSpecifications/LicenseSpecificationsItem/LicenseConfigurationArn":
      arnProp("AWS::LicenseManager::LicenseConfiguration"),
  },
  "AWS::EC2::NetworkInterface": {
    "GroupSet/GroupSetItem": suggest("AWS::EC2::SecurityGroup", "GroupId"),
  },
  "AWS::EC2::Route": {
    GatewayId: [
      suggest("AWS::EC2::InternetGateway", "InternetGatewayId"),
      suggest("AWS::EC2::VPNGateway", "VPNGatewayId"),
    ],
  },
  "AWS::EC2::VPCCidrBlock": {
    Ipv4IpamPoolId: suggest("AWS::EC2::IPAMPool", "IpamPoolId"),
    Ipv6IpamPoolId: suggest("AWS::EC2::IPAMPool", "IpamPoolId"),
  },
  "AWS::EC2::VPCEndpointConnectionNotification": {
    ConnectionNotificationArn: arnProp("AWS::SNS::Topic", "TopicArn"),
  },

  // AWS::ECS
  "AWS::ECS::TaskDefinition": {
    "ContainerDefinitions/ContainerDefinitionsItem/Secrets/SecretsItem/ValueFrom":
      [
        suggest("AWS::SecretsManager::Secret", "Id"),
        suggest("AWS::SSM::Parameter", "/domain/Name"),
      ],
  },

  // AWS::ElasticLoadBalancingV2
  "AWS::ElasticLoadBalancingV2::TargetGroup": {
    "Targets/TargetsItem/Id": [
      suggest("AWS::EC2::Instance", "InstanceId"),
      suggest("AWS::Lambda::Function", "Arn"),
      suggest("AWS::ElasticLoadBalancingV2::LoadBalancer", "LoadBalancerArn"),
    ],
  },

  // AWS::KMS
  "AWS::KMS::Key": {
    KeyPolicy: policyDocumentProp,
  },

  // AWS::Logs
  "AWS::Logs::LogGroup": {
    DataProtectionPolicy: policyDocumentProp,
  },

  // Props that exist on resources across all of AWS
  ".*": {
    // Policy document props
    ".*(PolicyDocument|PolicyText)": policyDocumentProp,

    // AWS::EC2 ARNs/IDs/Versions
    ".*IpamArn": arnProp("AWS::EC2::IPAM"),
    ".*IpamPoolArn": arnProp("AWS::EC2::IPAMPool"),
    ".*IpamScopeArn": arnProp("AWS::EC2::IPAMScope"),
    ".*LaunchTemplate*/Version": [
      suggest("AWS::EC2::LaunchTemplate", "LatestVersionNumber"),
      suggest("AWS::EC2::LaunchTemplate", "DefaultVersionNumber"),
    ],
    ".*LocalGatewayRouteTableArn": arnProp(
      "AWS::EC2::LocalGatewayRouteTable",
      "LocalGatewayRouteTableArn",
    ),
    ".*GroupId": suggest("AWS::EC2::SecurityGroup", "GroupId"),
    ".*GroupName": suggest("AWS::EC2::SecurityGroup", "/domain/GroupName"),

    // AWS::ElasticLoadBalancingV2 ARNs
    ".*LoadBalancerArn": arnProp(
      "AWS::ElasticLoadBalancingV2::LoadBalancer",
      "LoadBalancerArn",
    ),
    ".*(TargetGroupArn|TargetGroup/Arn)": arnProp(
      "AWS::ElasticLoadBalancingV2::TargetGroup",
      "TargetGroupArn",
    ),

    // AWS::KMS ARNs
    ".*KmsKeyArn": arnProp("AWS::KMS::Key"),

    // AWS::IAM ARNs
    ".*(InstanceProfileArn|InstanceProfile/Arn|InstanceProfileSpecification/Arn)":
      arnProp("AWS::IAM::InstanceProfile"),
    ".*RoleArn": arnProp("AWS::IAM::Role"),
    ".*SAMLProviderArn": arnProp("AWS::IAM::SAMLProvider"),

    // AWS::Lambda ARNs
    ".*(LambdaArn|LambdaFunctionArn)": arnProp("AWS::Lambda::Function"),

    // AWS::LicenseManager ARNs
    ".*(LicenseArn|LicenseConfigurationArn)": arnProp(
      "AWS::LicenseManager::License",
      "LicenseArn",
    ),

    // AWS::Logs ARNs
    ".*LogGroupArn": arnProp("AWS::Logs::LogGroup"),

    // AWS::NetworkManager ARNs
    ".*CoreNetworkArn": arnProp(
      "AWS::NetworkManager::CoreNetwork",
      "CoreNetworkArn",
    ),

    // AWS::RDS ARNs
    ".*DbInstanceArn": arnProp("AWS::RDS::DBInstance", "DBInstanceArn"),
    ".*DbClusterArn": arnProp("AWS::RDS::DBCluster", "DBClusterArn"),
    ".*DbProxyArn": arnProp("AWS::RDS::DBProxy", "DBProxyArn"),

    // AWS::ResourceGroups ARNs
    ".*ResourceGroupArn": arnProp("AWS::ResourceGroups::Group"),

    // AWS::SNS ARNs
    ".*TopicArn": arnProp("AWS::SNS::Topic", "TopicArn"),

    // AWS::VpcLattice ARNs
    ".*ResourceConfigurationArn": arnProp(
      "AWS::VpcLattice::ResourceConfiguration",
    ),
    ".*ServiceNetworkArn": arnProp("AWS::VpcLattice::ServiceNetwork"),
  },
};

// AWS-specific schema overrides
export const AWS_SCHEMA_OVERRIDES = new Map<string, SchemaOverrideFn>([
  [
    "AWS::EC2::Instance",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];

      const overrideUserDataAttributeFuncCode = Deno.readTextFileSync(
        "./src/pipelines/aws/funcs/overrides/AWS::EC2::Instance/attribute/base64EncodeUserData.ts",
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
          "./src/pipelines/aws/funcs/overrides/AWS::EC2::Instance/actions/reboot.ts",
          "Reboot Ec2 Instance",
          "other",
          "5e38470604abb5c3ccc2ab60b31c5c0a05e9b381a2db73a15f4f8d55ec441bbd",
        );
      spec.funcs.push(rebootFunc);
      variant.actionFuncs.push(rebootActionFuncSpec);

      // Create the Stop Action
      const { func: stopFunc, actionFuncSpec: stopActionFuncSpec } =
        attachExtraActionFunction(
          "./src/pipelines/aws/funcs/overrides/AWS::EC2::Instance/actions/stop.ts",
          "Stop Ec2 Instance",
          "other",
          "de2c03b1caff5e7a1011a8c0ac6dc6dc99af77d15d0bc1f93e7c4eb9d7307f22",
        );
      spec.funcs.push(stopFunc);
      variant.actionFuncs.push(stopActionFuncSpec);

      // Create the Start Action
      const { func: startFunc, actionFuncSpec: startActionFuncSpec } =
        attachExtraActionFunction(
          "./src/pipelines/aws/funcs/overrides/AWS::EC2::Instance/actions/start.ts",
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
        "./src/pipelines/aws/funcs/overrides/AWS::EC2::LaunchTemplate/import.ts";

      modifyFunc(spec, importTargetId, newImportId, importPath);

      const discoverTargetId = MANAGEMENT_FUNCS["Discover on AWS"].id;
      const newDiscoverId =
        "cfebba8fc2d7cd88e5fc2b0c47a777b3737b8c2bcb88fbbb143be48018f22836";
      const discoverPath =
        "./src/pipelines/aws/funcs/overrides/AWS::EC2::LaunchTemplate/discover.ts";

      modifyFunc(spec, discoverTargetId, newDiscoverId, discoverPath);
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
    },
  ],
  [
    "AWS::EC2::SecurityGroupIngress",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];
      const domainId = variant.domain.uniqueId;

      if (!domainId) return;

      const { func, leafFuncSpec } = attachQualificationFunction(
        "./src/pipelines/aws/funcs/overrides/AWS::EC2::SecurityGroupIngress/qualifications/checkForEitherGroupIdOrGroupName.ts",
        "GroupId OR GroupName",
        "23f026310223509f053b55bfa386772eecc2d00e3090dbeb65766ac63f8c53a2",
        domainId,
      );

      spec.funcs.push(func);
      variant.leafFunctions.push(leafFuncSpec);
    },
  ],
  [
    "AWS::AutoScaling::AutoScalingGroup",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];

      // Modify the existing update function instead of replacing it
      const updateTargetId = ACTION_FUNC_SPECS["Update Asset"].id;
      const newUpdateId =
        "c7e6bf82e9d7fa438f6a9151a1b1f4c6f4b18ae50eacf462bc81d2b31278e1c5";
      const updatePath =
        "./src/pipelines/aws/funcs/overrides/AWS::AutoScaling::AutoScalingGroup/actions/awsCloudControlUpdate.ts";
      modifyFunc(spec, updateTargetId, newUpdateId, updatePath);

      const {
        func: refreshInstancesFunc,
        actionFuncSpec: refreshInstancesFuncSpec,
      } = attachExtraActionFunction(
        "./src/pipelines/aws/funcs/overrides/AWS::AutoScaling::AutoScalingGroup/actions/instanceRefresh.ts",
        "Refresh Autoscaling Group Instances",
        "other",
        "300d62f40cb1268e6f4cd2320be8c373da7f148d0d9e6e69d1b2879202794b5f",
      );
      spec.funcs.push(refreshInstancesFunc);
      variant.actionFuncs.push(refreshInstancesFuncSpec);
    },
  ],
  [
    "AWS::ImageBuilder::Component",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];

      const prop = propForOverride(variant.domain, "Data");
      prop!.data.widgetKind = "CodeEditor";
    },
  ],
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
      if (!propUsageMapProp.data?.defaultValue) {
        throw new Error("Prop has no default value");
      }

      const defaultValue = JSON.parse(
        propUsageMapProp.data.defaultValue as string,
      );
      let createOnly = defaultValue["createOnly"];
      const updatable = defaultValue["updatable"];

      propsToRemove.forEach((propName) => {
        const prop = propForOverride(variant.domain, propName);

        const currentWidgetOptions = prop.data.widgetOptions;
        prop.data.widgetOptions = currentWidgetOptions?.filter(
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
        "./src/pipelines/aws/funcs/overrides/AWS::ECS::TaskDefinition/actions/update.ts";
      modifyFunc(spec, updateTargetId, newUpdateId, newUpdatePath);
    },
  ],
  [
    "AWS::EKS::Nodegroup",
    (_spec: ExpandedPkgSpec) => {
      // Placeholder - no current overrides for this schema
    },
  ],
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
        "./src/pipelines/aws/funcs/overrides/AWS::EC2::VPNConnection/actions/refresh.ts";
      modifyFunc(spec, refreshTargetId, newRefreshId, refreshPath);
    },
  ],
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
]);