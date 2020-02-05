import { arg, objectType, inputObjectType } from "nexus";
import { MQTTPubSub } from "@/mqtt-pubsub/mqtt-pubsub";

import { Message } from "protobufjs";
import { protobufLoader } from "@/protobuf";
import { NexusGenRootTypes, NexusGenArgTypes } from "@/fullstack-typegen";

const pubsub = new MQTTPubSub({ rawData: true });

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
        const messageType = protobufLoader.root.lookupType(
          "si.ssh_key.EntityEvent",
        );
        const response = messageType.decode(payload["message"]);
        return response as NexusGenRootTypes["SshKeyEntityEvent"];
      },
      // @ts-ignore - We know it doesn't exist, but it works anyway
      subscribe: (
        // @ts-ignore - we know, its any.
        _,
        args: NexusGenArgTypes["Subscription"]["streamEntityEvents"],
      ) => {
        return pubsub.asyncIterator(
          `+/+/${args.input.workspaceId}/+/+/+/action/+/+/result`,
        );
      },
    });
  },
});

export const subscriptionTypes = [subscription, StreamEntityEventsRequest];
