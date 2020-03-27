import { Metadata } from "grpc";
import { arg, objectType, inputObjectType, interfaceType } from "nexus";
import _ from "lodash";

import { MQTTPubSub } from "@/mqtt-pubsub/mqtt-pubsub";
import { protobufLoader } from "@/protobuf";
import { NexusGenRootTypes, NexusGenArgTypes } from "@/fullstack-typegen";
import { logger } from "@/logger";
import { environment } from "@/environment";

const pubsub = new MQTTPubSub({
  brokerUrl: environment.mqttBrokerUrl,
  rawData: true,
});

const EntityEvent = interfaceType({
  name: "EntityEvent",
  definition(t) {
    t.id("id");
    t.list.string("tenantIds");
    t.string("naturalKey");
    t.string("typeName");
    t.string("userId");
    t.string("actionName");
    t.list.string("outputLines");
    t.string("createTime");
    t.string("updatedTime");
    t.string("finalTime");
    t.boolean("finalized");
    t.string("entityId");
    t.string("componentId");
    t.string("integrationId");
    t.string("integrationServiceId");
    t.string("workspaceId");
    t.string("organizationId");
    t.string("billingAccountId");
    t.boolean("success");
    t.string("errorMessage");
    t.list.string("errorLines");
    // @ts-ignore - this is generated, I know, but you gotta deal with it
    t.resolveType(async function resolveType(source, _context, _info) {
      const pascalCaseType = _.upperFirst(_.camelCase(source.typeName));
      return pascalCaseType;
    });
  },
});

const StreamEntityEventsRequest = inputObjectType({
  name: "StreamEntityEventsRequest",
  definition(t) {
    t.string("scopeByTenantId", { required: true });
  },
});

const subscription = objectType({
  name: "Subscription",
  definition(t) {
    t.field("streamEntityEvents", {
      type: "EntityEvent",
      args: {
        input: arg({ type: "StreamEntityEventsRequest", required: true }),
      },
      resolve: payload => {
        logger.log("warn", "resolving item", { payload });
        const topicParts = payload["topic"].split("/");
        const entityIdIndex = topicParts.length - 5;
        const entityId = topicParts[entityIdIndex];
        const entityExtractRe = new RegExp("(.+)_(.+)");
        const entityExtractResult = entityId.match(entityExtractRe);
        logger.log("warn", "about to pick", {
          entityExtractResult,
          entityId,
          entityIdIndex,
          topicParts,
        });

        const protobufType = `si.${entityExtractResult[1]}.EntityEvent`;
        logger.log("warn", "about to deserialize", {
          protobufType,
        });

        const messageType = protobufLoader.root.lookupType(protobufType);
        const response = messageType.decode(payload["message"]);
        return response as NexusGenRootTypes["EntityEvent"];
      },
      // @ts-ignore - We know it doesn't exist, but it works anyway
      subscribe: (
        // @ts-ignore - we know, its any.
        _,
        args: NexusGenArgTypes["Subscription"]["streamEntityEvents"],
      ) => {
        const input = args.input;
        // WARNING: THIS NEEDS AUTHZ!
        // WARNING: THIS NEEDS WAY MORE INPUT VALIDATION
        if (input.scopeByTenantId.startsWith("billing_account")) {
          return pubsub.asyncIterator(
            `${input.scopeByTenantId}/+/+/+/+/+/action/+/+/result`,
          );
        } else if (input.scopeByTenantId.startsWith("organization")) {
          return pubsub.asyncIterator(
            `+/${input.scopeByTenantId}/+/+/+/+/action/+/+/result`,
          );
        } else if (input.scopeByTenantId.startsWith("workspace")) {
          return pubsub.asyncIterator(
            `+/+/${input.scopeByTenantId}/+/+/+/action/+/+/result`,
          );
        } else if (input.scopeByTenantId.split(":")[0].endsWith("_entity")) {
          return pubsub.asyncIterator(
            `+/+/+/+/+/${input.scopeByTenantId}/action/+/+/result`,
          );
        } else if (
          input.scopeByTenantId.split(":")[0].endsWith("_entity_event")
        ) {
          return pubsub.asyncIterator(
            `+/+/+/+/+/+/action/+/${input.scopeByTenantId}/result`,
          );
        } else {
          throw "invalid tenant id in scopeByTenantId";
        }
      },
    });
  },
});

export const subscriptionTypes = [
  subscription,
  StreamEntityEventsRequest,
  EntityEvent,
];
