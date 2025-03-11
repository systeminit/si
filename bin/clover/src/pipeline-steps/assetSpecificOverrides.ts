import _ from "npm:lodash";
import _logger from "../logger.ts";
import {
  createInputSocketFromProp,
  ExpandedSocketSpec,
  setAnnotationOnSocket,
} from "../spec/sockets.ts";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";
import {
  createScalarProp,
  ExpandedPropSpec,
  ExpandedPropSpecFor,
  findPropByName,
} from "../spec/props.ts";
import { PropUsageMap } from "./addDefaultPropsAndSockets.ts";
import { MANAGEMENT_FUNCS, modifyFunc } from "../spec/funcs.ts";

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
  ["ContainerDefinitions Secrets", (spec: ExpandedPkgSpec) => {
    const variant = spec.schemas[0].variants[0];

    const prop = propForOverride(variant.domain, "ValueFrom");
    if (!prop) return;
    const socket = createInputSocketFromProp(prop);
    setAnnotationOnSocket(socket, { tokens: ["Id"] });
    variant.sockets.push(socket);
  }],
  ["AWS::EC2::Instance", (spec: ExpandedPkgSpec) => {
    const variant = spec.schemas[0].variants[0];

    const prop = propForOverride(variant.domain, "UserData");
    if (!prop) return;
    prop!.data.widgetKind = "CodeEditor";
    
    const launchTemplateProp = propForOverride(variant.domain, "LaunchTemplate");
    if (!launchTemplateProp || launchTemplateProp.kind !== "object") return;
    
    const launchTemplateNameProp = propForOverride(launchTemplateProp, "LaunchTemplateName");
    if (!launchTemplateNameProp) return;
    
    const launchTemplateNameSocket = createInputSocketFromProp(
      launchTemplateNameProp,
      [
        { tokens: ["Launch Template Name"] },
        { tokens: ["LaunchTemplateName"] },
        { tokens: ["LaunchTemplateName<string<scalar>>"] },
      ],
      "Launch Template Name",
    );
    variant.sockets.push(launchTemplateNameSocket);
  }],
  ["AWS::EC2::LaunchTemplate", (spec: ExpandedPkgSpec) => {
    const variant = spec.schemas[0].variants[0];
    
    // Ensures we can connect to the Version input for the EC2 Instance and AutoScaling Group Assets
    const defaultVersionSocket = variant.sockets.find(
      (s: ExpandedSocketSpec) => s.name === "Default Version Number" && s.data.kind === "output",
    );
    if (!defaultVersionSocket) return;
    setAnnotationOnSocket(defaultVersionSocket, { tokens: ["Version"] });
    
    const latestVersionSocket = variant.sockets.find(
      (s: ExpandedSocketSpec) => s.name === "Latest Version Number" && s.data.kind === "output",
    );
    if (!latestVersionSocket) return;
    setAnnotationOnSocket(latestVersionSocket, { tokens: ["Version"] });
    
    const ltData = propForOverride(variant.domain, "LaunchTemplateData");
    if (!ltData || ltData.kind !== "object") return;

    const prop = propForOverride(ltData, "UserData");
    if (!prop) return;

    const socket = createInputSocketFromProp(prop);
    variant.sockets.push(socket);
    prop!.data.widgetKind = "CodeEditor";

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
  }],
  ["AWS::EC2::NetworkInterface", (spec: ExpandedPkgSpec) => {
    const variant = spec.schemas[0].variants[0];

    // Add an annotation for the Id output socket to connect to HostedZoneId
    const socket = variant.sockets.find(
      (s: ExpandedSocketSpec) => s.name === "Id" && s.data.kind === "output",
    );
    if (!socket) return;

    setAnnotationOnSocket(socket, { tokens: ["NetworkInterfaceId"] });

    const prop = propForOverride(variant.domain, "GroupSet");
    if (!prop) return;
    const groupSocket = createInputSocketFromProp(prop);

    setAnnotationOnSocket(groupSocket, { tokens: ["GroupId"] });
    variant.sockets.push(groupSocket);
  }],
  ["AWS::EC2::Route", (spec: ExpandedPkgSpec) => {
    const variant = spec.schemas[0].variants[0];

    const prop = propForOverride(variant.domain, "GatewayId");
    if (!prop) return;
    const socket = createInputSocketFromProp(prop);

    setAnnotationOnSocket(socket, { tokens: ["InternetGatewayId"] });
    setAnnotationOnSocket(socket, { tokens: ["VPNGatewayId"] });

    variant.sockets.push(socket);
  }],
  ["AWS::EC2::VPCEndpoint", (spec: ExpandedPkgSpec) => {
    const variant = spec.schemas[0].variants[0];

    const prop = propForOverride(variant.domain, "PolicyDocument");
    if (!prop) return;
    prop.kind = "json";
    prop!.data.widgetKind = "CodeEditor";
  }],
  ["AWS::KMS::Key", (spec: ExpandedPkgSpec) => {
    const variant = spec.schemas[0].variants[0];

    const prop = propForOverride(variant.domain, "KeyPolicy");
    if (!prop) return;
    prop.kind = "json";
    prop!.data.widgetKind = "CodeEditor";
  }],
  ["AWS::Logs::LogGroup", (spec: ExpandedPkgSpec) => {
    const variant = spec.schemas[0].variants[0];

    const prop = propForOverride(variant.domain, "DataProtectionPolicy");
    if (!prop) return;
    prop.kind = "json";
    prop!.data.widgetKind = "CodeEditor";
  }],
  ["AWS::RDS::DBParameterGroup", (spec: ExpandedPkgSpec) => {
    const variant = spec.schemas[0].variants[0];

    const prop = propForOverride(variant.domain, "Parameters");
    if (!prop) return;
    prop.kind = "map";

    if (prop.kind === "map") {
      prop.typeProp = createScalarProp(
        "parameter",
        "string",
        prop.metadata.propPath,
        false,
      );
    }
  }],
  [
    "AWS::SecretsManager::Secret",
    addSecretProp("Secret String", "secretString", ["SecretString"]),
  ],
  ["AWS::EC2::NetworkInterface", (spec: ExpandedPkgSpec) => {
    const variant = spec.schemas[0].variants[0];

    const prop = propForOverride(variant.domain, "GroupSet");
    if (!prop) return;
    const groupSocket = createInputSocketFromProp(prop);

    setAnnotationOnSocket(groupSocket, { tokens: ["GroupId"] });
    variant.sockets.push(groupSocket);
  }],
  ["AWS::AutoScaling::AutoScalingGroup", (spec: ExpandedPkgSpec) => {
    const variant = spec.schemas[0].variants[0];
    const launchTemplateProp = propForOverride(
      variant.domain,
      "LaunchTemplate",
    );
    if (!launchTemplateProp || launchTemplateProp.kind !== "object") return;

    const launchTemplateName = propForOverride(
      launchTemplateProp,
      "LaunchTemplateName",
    );
    if (!launchTemplateName) return;
    const launchTemplateNameSocket = createInputSocketFromProp(
      launchTemplateName,
      [
        { tokens: ["Launch Template Name"] },
        { tokens: ["LaunchTemplateName"] },
        { tokens: ["LaunchTemplateName<string<scalar>>"] },
      ],
      "Launch Template Name",
    );
    variant.sockets.push(launchTemplateNameSocket);

    const launchTemplateId = propForOverride(
      launchTemplateProp,
      "LaunchTemplateId",
    );
    if (!launchTemplateId) return;

    const launchTemplateIdSocket = createInputSocketFromProp(launchTemplateId, [
      { tokens: ["Launch Template Id"] },
      { tokens: ["LaunchTemplateId"] },
      { tokens: ["LaunchTemplateId<string<scalar>>"] },
    ], "Launch Template Id");
    variant.sockets.push(launchTemplateIdSocket);

    const launchTemplateVersion = propForOverride(
      launchTemplateProp,
      "Version",
    );
    if (!launchTemplateVersion) return;

    const launchTemplateVersionSocket = createInputSocketFromProp(
      launchTemplateVersion,
      [
        { tokens: ["Launch Template Version"] },
        { tokens: ["LaunchTemplateVersion"] },
        { tokens: ["LaunchTemplateVersion<string<scalar>>"] },
        { tokens: ["DefaultVersionNumber"] },
        { tokens: ["LatestVersionNumber"] },
      ],
      "Launch Template Version",
    );
    variant.sockets.push(launchTemplateVersionSocket);
  }],
  ["TargetGroup Targets", (spec: ExpandedPkgSpec) => {
    const variant = spec.schemas[0].variants[0];

    // Add an annotation for the Id output socket to connect to HostedZoneId
    const socket = variant.sockets.find(
      (s: ExpandedSocketSpec) => s.name === "Id" && s.data.kind === "input",
    );
    if (!socket) return;

    setAnnotationOnSocket(socket, { tokens: ["InstanceId"] });
    setAnnotationOnSocket(socket, { tokens: ["arn", "string"] });
    setAnnotationOnSocket(socket, { tokens: ["arn"] });
  }],
  ["AWS::ImageBuilder::Component", (spec: ExpandedPkgSpec) => {
    const variant = spec.schemas[0].variants[0];

    const prop = propForOverride(variant.domain, "Data");
    if (!prop) return;
    prop!.data.widgetKind = "CodeEditor";
  }],
  ["AWS::EC2::VPCCidrBlock", (spec: ExpandedPkgSpec) => {
    const variant = spec.schemas[0].variants[0];

    const ipv6IpamProp = propForOverride(variant.domain, "Ipv6IpamPoolId");
    if (!ipv6IpamProp) return;

    const ipv6IpamSocket = createInputSocketFromProp(ipv6IpamProp, [
      { tokens: ["Ipam Pool Id"] },
      { tokens: ["IpamPoolId"] },
      { tokens: ["IpamPoolId", "string", "scalar"] },
    ]);
    variant.sockets.push(ipv6IpamSocket);
    const ipv4IpamProp = propForOverride(variant.domain, "Ipv4IpamPoolId");
    if (!ipv4IpamProp) return;

    const ipv4IpamSocket = createInputSocketFromProp(ipv4IpamProp, [
      { tokens: ["Ipam Pool Id"] },
      { tokens: ["IpamPoolId"] },
      { tokens: ["IpamPoolId", "string", "scalar"] },
    ]);
    variant.sockets.push(ipv4IpamSocket);
  }],
  ["AWS::ECS::Cluster", (spec: ExpandedPkgSpec) => {
    const variant = spec.schemas[0].variants[0];
    const configurationProp = propForOverride(variant.domain, "Configuration");
    if (!configurationProp || configurationProp.kind !== "object") return;

    const managedStorageConfigurationProp = propForOverride(
      configurationProp,
      "ManagedStorageConfiguration",
    );
    if (
      !managedStorageConfigurationProp ||
      managedStorageConfigurationProp.kind !== "object"
    ) return;

    const fargateKmsProp = propForOverride(
      managedStorageConfigurationProp,
      "FargateEphemeralStorageKmsKeyId",
    );
    if (!fargateKmsProp) return;

    const fargateKmsSocket = createInputSocketFromProp(fargateKmsProp, [
      { tokens: ["Key Id"] },
      { tokens: ["KeyId"] },
      { tokens: ["KeyId", "string", "scalar"] },
    ], "Fargate Storage KMS Key Id");
    variant.sockets.push(fargateKmsSocket);

    const kmsKeyIdProp = propForOverride(
      managedStorageConfigurationProp,
      "KmsKeyId",
    );
    if (!kmsKeyIdProp) return;

    const kmsKeyIdSocket = createInputSocketFromProp(kmsKeyIdProp, [
      { tokens: ["Key Id"] },
      { tokens: ["KeyId"] },
      { tokens: ["KeyId", "string", "scalar"] },
    ], "Storage KMS Key Id");
    variant.sockets.push(kmsKeyIdSocket);
  }],
]);

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
    const propUsageMapProp = extraProp.entries.find((p) =>
      p.name === "PropUsageMap"
    );
    const propUsageMap = JSON.parse(
      propUsageMapProp?.data.defaultValue,
    ) as PropUsageMap;

    if (!propUsageMapProp || !Array.isArray(propUsageMap?.secrets)) {
      return;
    }

    // Remove secret from the domain tree
    secretParent.entries = secretParent.entries.filter((
      p: ExpandedPropSpec,
    ) => p.name !== secretName);

    // Add prop to secrets tree
    secretProp.data.widgetKind = "Secret";
    secretProp.data.widgetOptions = [{
      "label": "secretKind",
      "value": secretKind,
    }];
    variant.secrets.entries.push(secretProp);
    // Replace "domain" with "secrets" on propPath
    secretProp.metadata.propPath[1] = "secrets";

    // Add socket for secret prop
    const secretStringProp = createInputSocketFromProp(secretProp);
    variant.sockets.push(secretStringProp);
    setAnnotationOnSocket(secretStringProp, { tokens: [secretKind] });

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
): ExpandedPropSpec | undefined {
  const prop = findPropByName(objPropSpec, propName);
  if (!prop) {
    logger.warn(`Prop not found for override ${propName}!`);
  }
  return prop;
}
