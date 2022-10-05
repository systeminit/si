import { defineStore } from "pinia";
import _ from "lodash";
import ReconnectingWebSocket from "reconnecting-websocket";
import { computed, reactive, ref, watch } from "vue";
import { API_WS_URL } from "@/utils/api";
import { WsEventService } from "@/service/ws_event";
import { useAuthStore } from "../auth.store";
import { WsEventPayloadMap } from "./realtime_events";

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
    callback: (value: WsEventPayloadMap[K]) => unknown;
  };
}[keyof WsEventPayloadMap];

type TrackedSubscription = EventTypeAndCallback & {
  id: SubscriptionId;
  topic: SubscriptionTopic;
  subscriberId: SubscriberId;
};

export const useRealtimeStore = defineStore("realtime", () => {
  const authStore = useAuthStore();

  // ReconnectingWebsocket is a small wrapper around the native Websocket that should
  // handle basic reconnection logic
  const socket = new ReconnectingWebSocket(
    () =>
      `${API_WS_URL}/billing_account_updates?token=Bearer+${authStore.token}`,
    [],
    {
      // see options https://www.npmjs.com/package/reconnecting-websocket#available-options
      startClosed: true, // don't start connected - we'll watch auth to trigger
      // TODO: tweak settings around reconnection behaviour
    },
  );

  // boolean tracking whether we are expecting connection to be active
  // currently only logic is if user is logged in
  const connectionShouldBeEnabled = computed(() => authStore.userIsLoggedIn);

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
    topicSubscriptionCounter[sub.topic]--;
    if (topicSubscriptionCounter[sub.topic] === 0) {
      // TODO: send topic unsubscribe message to server
    }
    delete subscriptions[sub.id];
  }

  // TODO: add optional arg to unsubscribe to specific event types, topics, or by subscription id
  function unsubscribe(subscriberId: SubscriberId) {
    _.each(subscriptionsBySubscriberId.value[subscriberId], (sub) => {
      destroySingleSubscription(sub.id);
    });
  }

  socket.addEventListener("message", (messageEvent) => {
    const messageEventData = JSON.parse(messageEvent.data);

    console.log("ws message!", messageEventData);

    _.each(subscriptions, (sub) => {
      // TODO: also filter by topic once we receive this info from the backend
      if (sub.eventType === messageEventData.payload.kind) {
        // we may also need the version and history actor, can pass through as second arg
        sub.callback(messageEventData.payload.data);
      }
    });

    // dispatch events back to rxjs services
    // TODO: will remove once everything moved over
    WsEventService.dispatch(messageEventData);
  });
  socket.addEventListener("error", (errorEvent) => {
    console.log("ws error", errorEvent.error, errorEvent.message);
  });

  return {
    connectionStatus,
    // subscriptions, // can expose here to show in devtools
    subscribe,
    unsubscribe,
  };
});
