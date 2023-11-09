import { defineStore } from "pinia";
import * as _ from "lodash-es";
import ReconnectingWebSocket from "reconnecting-websocket";
import { computed, reactive, ref, watch } from "vue";
import { API_WS_URL } from "@/store/apis";
import { ActorView } from "@/api/sdf/dal/history_actor";
import { useAuthStore } from "../auth.store";
import { WebsocketRequest, WsEventPayloadMap } from "./realtime_events";

type RawConnectionStatus = "open" | "closed";

type SubscriptionId = string;
type SubscriberId = string;
type SubscriptionTopic = string;

// some fairly magic TS wizardry happening here...
// just reshuffling the WsEventPayloadMap into a format usable in our subscribe call
// idea from https://stackoverflow.com/questions/68304361/how-to-define-an-array-of-generic-objects-in-typescript-each-item-having-a-diff
type EventTypeAndCallback = {
  [K in keyof WsEventPayloadMap]: {
    eventType: K;
    callback: (
      payload: WsEventPayloadMap[K],
      metadata: RealtimeEventMetadata,
    ) => unknown;
  };
}[keyof WsEventPayloadMap];

type TrackedSubscription = EventTypeAndCallback & {
  id: SubscriptionId;
  topic: SubscriptionTopic;
  subscriberId: SubscriberId;
};

// shape of the extra data that comes through the websocket along with the payload
type RealtimeEventMetadata = {
  version: number;
  workspace_pk: string;
  actor: ActorView;
};

export const useRealtimeStore = defineStore("realtime", () => {
  const authStore = useAuthStore();

  // TODO: need to think about how websockets multiple workspaces

  // ReconnectingWebsocket is a small wrapper around the native Websocket that should
  // handle basic reconnection logic
  const socket = new ReconnectingWebSocket(
    () =>
      `${API_WS_URL}/workspace_updates?token=Bearer+${authStore.selectedWorkspaceToken}`,
    [],
    {
      // see options https://www.npmjs.com/package/reconnecting-websocket#available-options
      startClosed: true, // don't start connected - we'll watch auth to trigger
      // TODO: tweak settings around reconnection behaviour
    },
  );

  // boolean tracking whether we are expecting connection to be active
  // currently only logic is if user is logged in
  const connectionShouldBeEnabled = computed(
    () =>
      authStore.userIsLoggedInAndInitialized &&
      authStore.selectedWorkspaceToken,
  );

  // trigger connect / close as necessary
  watch(
    connectionShouldBeEnabled,
    () => {
      if (connectionShouldBeEnabled.value) socket.reconnect();
      else socket.close();
    },
    { immediate: true },
  );

  const rawConnectionStatus = ref("closed" as RawConnectionStatus);

  socket.addEventListener("open", () => {
    rawConnectionStatus.value = "open";
  });
  socket.addEventListener("close", () => {
    rawConnectionStatus.value = "closed";
  });

  // exposed connection status - useful to display in the UI
  const connectionStatus = computed(() => {
    // TODO: maybe use socket.readyState here, but not sure sufficient events are fired to see all states
    if (connectionShouldBeEnabled.value) {
      if (rawConnectionStatus.value === "open") return "connected";
      else return "disconnected";
      // TODO: could do better here differentiating between first connect vs reconnecting, etc
    } else {
      if (rawConnectionStatus.value === "open") return "closing";
      return "closed";
    }
  });

  // track subscriptions w/ topics, subscribers, etc
  let subCounter = 0;
  const topicSubscriptionCounter = {} as Record<SubscriptionTopic, number>;
  const subscriptions = reactive(
    {} as Record<SubscriptionId, TrackedSubscription>,
  );
  const subscriptionsBySubscriberId = computed(() =>
    _.groupBy(subscriptions, "subscriberId"),
  );

  function setupSingleSubscription(
    subscriberId: SubscriberId,
    topic: SubscriptionTopic,
    typeAndCallback: EventTypeAndCallback,
  ) {
    if (!topicSubscriptionCounter[topic]) {
      // TODO: send topic subscription message to server
      topicSubscriptionCounter[topic] = 0;
    }
    topicSubscriptionCounter[topic]++;

    const subscriptionId: SubscriptionId = [
      topic,
      typeAndCallback.eventType,
      subscriberId,
      subCounter++,
    ].join("%");

    subscriptions[subscriptionId] = {
      id: subscriptionId,
      subscriberId,
      topic,
      ...typeAndCallback,
    };

    return subscriptionId;
  }

  function subscribe(
    subscriberId: SubscriberId,
    topic: SubscriptionTopic,
    subscriptions: EventTypeAndCallback | EventTypeAndCallback[],
  ) {
    _.forEach(
      _.isArray(subscriptions) ? subscriptions : [subscriptions],
      (sub) => setupSingleSubscription(subscriberId, topic, sub),
    );
  }

  function destroySingleSubscription(id: SubscriptionId) {
    const sub = subscriptions[id];
    if (sub) {
      topicSubscriptionCounter[sub.topic]--;
      if (topicSubscriptionCounter[sub.topic] === 0) {
        // TODO: send topic unsubscribe message to server
      }
      delete subscriptions[sub.id];
    }
  }

  // TODO: add optional arg to unsubscribe to specific event types, topics, or by subscription id
  function unsubscribe(subscriberId: SubscriberId) {
    _.each(subscriptionsBySubscriberId.value[subscriberId], (sub) => {
      destroySingleSubscription(sub.id);
    });
  }

  function handleEvent(
    eventKind: string,
    eventData: any, // eslint-disable-line @typescript-eslint/no-explicit-any
    eventMetadata: RealtimeEventMetadata,
  ) {
    // Set the "VITE_LOG_WS" environment variable to true if you want to see logs for received WsEvents.
    if (import.meta.env.VITE_LOG_WS) {
      /* eslint-disable-next-line no-console */
      console.log("WS message", eventKind, eventData);
    }

    _.each(subscriptions, (sub) => {
      // TODO: also filter by topic once we receive this info from the backend
      if (sub.eventType === eventKind) {
        // TODO: probably want to convert the raw metadata into something easier to use
        // like a boolean that says whether this event came from the auth'd user
        sub.callback(eventData, eventMetadata);
      }
    });
  }

  const sendMessage = (req: WebsocketRequest) => {
    /* eslint-disable no-empty */
    try {
      socket.send(JSON.stringify(req));
    } catch {}
  };

  socket.addEventListener("message", (messageEvent) => {
    const messageEventData = JSON.parse(messageEvent.data);
    handleEvent(
      messageEventData.payload.kind,
      messageEventData.payload.data,
      _.omit(messageEventData, "payload") as RealtimeEventMetadata,
    );
  });
  socket.addEventListener("error", (errorEvent) => {
    /* eslint-disable-next-line no-console */
    console.log("ws error", errorEvent.error, errorEvent.message);
  });

  return {
    connectionStatus,
    sendMessage,
    // subscriptions, // can expose here to show in devtools
    subscribe,
    unsubscribe,
  };
});
