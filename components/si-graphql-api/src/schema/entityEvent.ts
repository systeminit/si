import { arg, objectType, inputObjectType } from "nexus";
import { MQTTPubSub } from "graphql-mqtt-subscriptions";
import { logger } from "@/logger";

import { environment } from "@/environment";
import { Context } from "@/.";
import { protobufLoader } from "@/protobuf";

const pubsub = new MQTTPubSub();

//        let topic = format!(
//            "{}/{}/{}/{}/{}/{}/{}/{}/{}/result",
//            entity_event.billing_account_id,
//            entity_event.organization_id,
//            entity_event.workspace_id,
//            entity_event.integration_id,
//            entity_event.integration_service_id,
//            entity_event.entity_id,
//            "action",
//            entity_event.action_name,
//            entity_event.id,
//        );

const StreamEntityEventsRequest = inputObjectType({
  name: "StreamEntityEventsRequest",
  definition(t) {
    t.string("workspaceId", { required: true });
  },
});

const subscription = objectType({
  name: "Subscription",
  definition(t) {
    t.field("streamEntityEvents", {
      type: "SshKeyEntityEvent",
      args: {
        input: arg({ type: "StreamEntityEventsRequest", required: true }),
      },
      resolve: payload => {
        logger.log("warn", "oh shit", { payload });
        const messageType = protobufLoader.root.lookupType(
          "si.ssh_key.EntityEvent",
        );
        const response = messageType.decode(Buffer.from(payload));
        logger.log("warn", "oh shit response", { response });
        return response;
      },
      subscribe: (_, args) =>
        pubsub.asyncIterator(
          `+/+/${args.input.workspaceId}/+/+/+/action/+/+/result`,
        ),
    });
  },
});

export const subscriptionTypes = [subscription, StreamEntityEventsRequest];
