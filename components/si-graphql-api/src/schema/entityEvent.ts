import { arg, objectType, inputObjectType } from "nexus";
import { MQTTPubSub } from "@/mqtt-pubsub/mqtt-pubsub";
import { logger } from "@/logger";

import { environment } from "@/environment";
import { Context } from "@/.";
import { protobufLoader } from "@/protobuf";

const pubsub = new MQTTPubSub({ rawData: true });

const StreamEntityEventsRequest = inputObjectType({
  name: "StreamEntityEventsRequest",
  definition(t) {
    t.string("workspaceId", { required: true });
  },
});

interface Payload {
  topic: string;
  message: Buffer;
}

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

//import { $$asyncIterator } from "iterall";
//
//export const withStaticFields = (
//  asyncIterator: AsyncIterator<any>,
//  staticFields: Record<string, any>,
//): Function => {
//  return (
//    rootValue: any,
//    args: any,
//    context: any,
//    info: any,
//  ): AsyncIterator<any> => {
//    return {
//      next() {
//        return asyncIterator.next().then(({ value, done }) => {
//          const messageType = protobufLoader.root.lookupType(
//            "si.ssh_key.EntityEvent",
//          );
//          const response = messageType.decode(Buffer.from(value));
//          console.log("error", "time is up", {
//            createTime: response["createTime"],
//          });
//          //logger.log("warn", "oh shit response", { response });
//          return {
//            value: response,
//            done,
//          };
//          //return {
//          //  value: {
//          //    ...value,
//          //    ...staticFields,
//          //  },
//          //  done,
//          //};
//        });
//      },
//      return() {
//        return Promise.resolve({ value: undefined, done: true });
//      },
//      throw(error) {
//        return Promise.reject(error);
//      },
//      [$$asyncIterator]() {
//        return this;
//      },
//    };
//  };
//};

export const subscriptionTypes = [subscription, StreamEntityEventsRequest];
