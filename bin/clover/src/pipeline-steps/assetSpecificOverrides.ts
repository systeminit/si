import _ from "npm:lodash";
import _logger from "../logger.ts";
import {
  createInputSocketFromProp,
  ExpandedSocketSpec,
  setAnnotationOnSocket,
} from "../spec/sockets.ts";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";
import { ExpandedPropSpec } from "../spec/props.ts";

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
  ["AWS::EC2::Route", (spec: ExpandedPkgSpec) => {
    // Add GatewayId Socket
    const variant = spec.schemas[0].variants[0];

    const prop = variant.domain.entries.find((p: ExpandedPropSpec) =>
      p.name === "GatewayId"
    );

    if (!prop) return;
    const socket = createInputSocketFromProp(prop);

    setAnnotationOnSocket(socket, { tokens: ["InternetGatewayId"] });
    setAnnotationOnSocket(socket, { tokens: ["VPNGatewayId"] });

    variant.sockets.push(socket);
  }],
  ["AWS::CloudTrail::Trail", (spec: ExpandedPkgSpec) => {
    const variant = spec.schemas[0].variants[0];

    for (const prop of variant.domain.entries) {
      switch (prop.name) {
        case "S3BucketName": {
          const socket = createInputSocketFromProp(prop, [{
            tokens: ["BucketName"],
          }]);

          variant.sockets.push(socket);
          break;
        }
        case "SnsTopicName": {
          const socket = createInputSocketFromProp(prop, [{
            tokens: ["TopicName"],
          }]);

          variant.sockets.push(socket);
          break;
        }
        case "KMSKeyId": {
          const socket = createInputSocketFromProp(prop, [{
            tokens: ["KeyId"],
          }]);

          variant.sockets.push(socket);
          break;
        }
      }
    }
  }],
  [
    "AWS::Route53::HostedZone",
    (spec: ExpandedPkgSpec) => {
      const variant = spec.schemas[0].variants[0];

      // Add an annotation for the Id output socket to connect to HostedZoneId
      const socket = variant.sockets.find(
        (s: ExpandedSocketSpec) => s.name === "Id" && s.data.kind === "output",
      );
      if (!socket) return;

      setAnnotationOnSocket(socket, { tokens: ["HostedZoneId"] });
    },
  ],
]);
