import { defineStore } from "pinia";
import * as _ from "lodash-es";
import ReconnectingWebSocket from "reconnecting-websocket";
import { computed, reactive, ref, watch } from "vue";
import { ulid } from "ulid";
import { API_WS_URL } from "@/store/apis.web";
import { omit } from "@/utils/omit";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { useAuthStore } from "../auth.store";
import { WebsocketRequest, WsEventPayloadMap } from "./realtime_events";

type RawConnectionStatus = "open" | "closed";

type SubscriptionId = string;
type SubscriberId = string;
type SubscriptionTopic = "all" | `workspace/${string}` | `changeset/${string}`;

// some fairly magic TS wizardry happening here...
// just reshuffling the WsEventPayloadMap into a format usable in our subscribe call
// idea from https://stackoverflow.com/questions/68304361/how-to-define-an-array-of-generic-objects-in-typescript-each-item-having-a-diff
type EventTypeAndCallback = {
  [K in keyof WsEventPayloadMap]: {
    eventType: K;
    debounce?: boolean | number;
    callback: (payload: WsEventPayloadMap[K], metadata: RealtimeEventMetadata) => unknown;
  };
}[keyof WsEventPayloadMap];

type TrackedSubscription = EventTypeAndCallback & {
  id: SubscriptionId;
  topic: SubscriptionTopic;
  subscriberId: SubscriberId;
};

type Actor = "System" | { User: string };

// shape of the extra data that comes through the websocket along with the payload
type RealtimeEventMetadata = {
  version: number;
  workspace_pk: string;
  actor: Actor;
  change_set_id: string;
  request_ulid: string; // the HTTP endpoint requestUlid that resulted in this event being fired
};

type EventKind = keyof WsEventPayloadMap;
type BufferedEvent = {
  ulid: string;
  eventKind: EventKind;
  payload: WsEventPayloadMap[EventKind];
  metadata: RealtimeEventMetadata;
  ttl: number;
};

export const useRealtimeStore = defineStore("realtime", () => {
  const authStore = useAuthStore();

  const bufferWatchList = reactive<ChangeSetId[]>([]);
  const wsEventBuffer = ref<Record<string, BufferedEvent>>({});
  const eventsRun = reactive<Map<string, string[]>>(new Map());

  // PSA: using map with reactive, because a record/{} that is reactive doesn't behave well with index-based operations
  const inflightRequests = reactive<Map<string, string>>(new Map()); // <requestUlid, API_NAME>

  // TODO: need to think about how websockets multiple workspaces

  // ReconnectingWebsocket is a small wrapper around the native Websocket that should
  // handle basic reconnection logic
  const socket = new ReconnectingWebSocket(
    () => `${API_WS_URL}/workspace_updates?token=Bearer+${authStore.selectedWorkspaceToken}`,
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
    () => authStore.userIsLoggedInAndInitialized && authStore.selectedWorkspaceToken,
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

  // TODO(johnrwatson): Fetching status from a public status page JSON representation
  // I have a DNS record set up for this but it's giving me grief, I'll come back and amend to
  // status-data.systeminit.com
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  async function fetchStatusPage(): Promise<any> {
    const response = await fetch("https://nhzefkyp7l.execute-api.us-east-1.amazonaws.com/data/payload.json");
    if (response.status === 200) {
      const data = await response.json();
      return data;
    }
  }

  // Custom sort function to sort incidents by severity
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  function sortIncidentsBySeverity(incidents?: any[]): any[] {
    const severityOrder: { [key: string]: number } = {
      MAINTENANCE: 4,
      UNAVAILABLE: 3,
      DEGRADED: 2,
      OPERATIONAL: 1,
    };

    return (
      incidents?.sort((a, b) => {
        const severityA = severityOrder[a.severitySlug?.toUpperCase()] || 5;
        const severityB = severityOrder[b.severitySlug?.toUpperCase()] || 5;
        return severityA - severityB;
      }) ?? []
    );
  }

  const applicationStatus = ref<string>("operational");

  // Check whether there is a degraded or outage state against the public statuspage
  const statusPageState = async () => {
    applicationStatus.value = "operational";

    try {
      const statusData = await fetchStatusPage();
      if (!statusData) {
        return;
      }

      const incidents = sortIncidentsBySeverity(statusData.incidents);

      // Loop through each incident after sorting
      for (const incident of incidents) {
        const resolvedTimestamp = incident.timestamps?.resolved;

        if (incident.components) {
          for (const component of incident.components) {
            // Check if the incident is unresolved and its severity is relevant
            if (
              !resolvedTimestamp &&
              ["UNAVAILABLE", "DEGRADED", "MAINTENANCE"].includes(component.condition.toUpperCase())
            ) {
              applicationStatus.value = component.condition.toLowerCase(); // Return the lowercased version of severity and break the loop
              break;
            }
          }
        }
      }
    } catch (error) {
      reportError(error);
    }
  };

  setInterval(statusPageState, 30 * 1000);

  // track subscriptions w/ topics, subscribers, etc
  let subCounter = 0;
  // const topicSubscriptionCounter = {} as Record<SubscriptionTopic, number>;
  const subscriptions = reactive({} as Record<SubscriptionId, TrackedSubscription>);
  const subscriptionsBySubscriberId = computed(() => _.groupBy(subscriptions, "subscriberId"));

  function setupSingleSubscription(
    subscriberId: SubscriberId,
    topic: SubscriptionTopic,
    typeAndCallback: EventTypeAndCallback,
  ) {
    /* if (!topicSubscriptionCounter[topic]) {
      // TODO: send topic subscription message to server
      topicSubscriptionCounter[topic] = 0;
    }
    topicSubscriptionCounter[topic]++; */

    const subscriptionId: SubscriptionId = [
      topic,
      typeAndCallback.eventType,
      subscriberId,
      subCounter++, // im not quite sure the value of this, with it, we need the `subscriptionsBySubscriberId` indirection
    ].join("%");

    const debounceMs = typeAndCallback.debounce === true ? 500 : typeAndCallback.debounce || 0;
    const wrappedCallback = debounceMs ? _.debounce(typeAndCallback.callback, debounceMs) : typeAndCallback.callback;

    subscriptions[subscriptionId] = {
      id: subscriptionId,
      subscriberId,
      topic,
      ...typeAndCallback,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      callback: wrappedCallback as any,
    };

    return subscriptionId;
  }

  function subscribe(
    subscriberId: SubscriberId,
    topic: SubscriptionTopic,
    _subscriptions: EventTypeAndCallback | EventTypeAndCallback[],
  ) {
    _.forEach(_.isArray(_subscriptions) ? _subscriptions : [_subscriptions], (sub) =>
      setupSingleSubscription(subscriberId, topic, sub),
    );

    // keys are IDs which are sortable, oldest first
    for (const evtId of Object.keys(wsEventBuffer.value).sort()) {
      const bufferedEvent = wsEventBuffer.value[evtId]!;
      if (bufferedEvent.ttl <= Date.now()) {
        // even though its not run, we're using this to clear out the data
        const topicsRun = eventsRun.get(bufferedEvent.ulid) || [];
        eventsRun.set(bufferedEvent.ulid, topicsRun);
        continue;
      }

      const topics = [
        `workspace/${bufferedEvent.metadata.workspace_pk}`,
        `changeset/${bufferedEvent.metadata.change_set_id}`,
      ];

      // support sending the same event to multiple subscribers
      Object.values(subscriptions).forEach((sub) => {
        const topicsRun = eventsRun.get(bufferedEvent.ulid) || [];
        if (
          sub.eventType === bufferedEvent.eventKind &&
          topics.includes(sub.topic) &&
          !topicsRun.includes(sub.topic) // don't run events twice for a given topic & subscribe call
        ) {
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          sub.callback(bufferedEvent.payload as any, bufferedEvent.metadata);
          topicsRun.push(sub.topic);
          eventsRun.set(bufferedEvent.ulid, topicsRun);
        }
      });
    }
  }

  function clearEventsRun() {
    let id;
    const keys = [...eventsRun.keys()];
    do {
      id = keys.shift();
      if (id) delete wsEventBuffer.value[id];
    } while (id);
    eventsRun.clear();
  }

  // clear out buffer data every 5 minutes
  setInterval(clearEventsRun, 1000 * 60 * 5);

  function destroySingleSubscription(id: SubscriptionId) {
    const sub = subscriptions[id];
    if (sub) {
      /* topicSubscriptionCounter[sub.topic] -= 1;
      if (topicSubscriptionCounter[sub.topic] === 0) {
        // TODO: send topic unsubscribe message to server
      } */
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
    // Set the "VITE_LOG_WS" environment variable to true if you want to see logs for received WsEvents (excluding cursor events).
    // Set the "VITE_LOG_WS_CURSOR" environment variable to true if you want to see logs for received cursor WsEvents.
    if (
      (import.meta.env.VITE_LOG_WS && !["Cursor", "Online"].includes(eventKind)) ||
      (import.meta.env.VITE_LOG_WS_CURSOR && eventKind === "Cursor") ||
      (import.meta.env.VITE_LOG_WS_ONLINE && eventKind === "Online")
    ) {
      /* eslint-disable-next-line no-console */
      console.log("WS message", eventKind, eventData, eventMetadata);
    }

    const topics: SubscriptionTopic[] = ["all", `workspace/${eventMetadata.workspace_pk}`];
    if (eventMetadata.change_set_id) {
      topics.push(`changeset/${eventMetadata.change_set_id}`);
    }
    // guaranteed to happen before data mutations in this changeset
    if (eventKind === "ChangeSetCreated") {
      if (eventMetadata.actor !== "System" && eventMetadata.actor.User === authStore.userPk) {
        bufferWatchList.push(eventData.changeSetId);
      }
    }
    if (eventKind === "ChangeSetApplied") {
      // applying a change set, we also want to notify people sitting on change sets
      // toRebaseChangeSetId is HEAD / where merges are going into
      try {
        topics.push(`changeset/${eventData.toRebaseChangeSetId}`);
      } catch (err) {
        // do nothing
      }
    }

    let dispatched = false;
    _.each(subscriptions, (sub) => {
      if (sub?.eventType === eventKind && topics.includes(sub.topic)) {
        sub.callback(eventData, eventMetadata);
        dispatched = true;
      }
    });
    if (!dispatched && eventKind !== "Cursor" && eventKind !== "Online") {
      if (
        // should we buffer an incoming event because we just created a changeset and the stores aren't set up yet?
        bufferWatchList.some((changeSetId) => eventMetadata.change_set_id === changeSetId)
      ) {
        const id = ulid();
        wsEventBuffer.value[id] = {
          ulid: id,
          eventKind: eventKind as keyof WsEventPayloadMap,
          payload: eventData,
          metadata: eventMetadata,
          ttl: Date.now() + 1 * 60 * 1000, // one minute from now
        };
      }
    }
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
      omit(messageEventData, "payload") as RealtimeEventMetadata,
    );
  });
  socket.addEventListener("error", (errorEvent) => {
    /* eslint-disable-next-line no-console */
    console.log("ws error", errorEvent.error, errorEvent.message);
  });

  return {
    applicationStatus,
    connectionStatus,
    sendMessage,
    // subscriptions, // can expose here to show in devtools
    subscribe,
    unsubscribe,
    inflightRequests,
  };
});
