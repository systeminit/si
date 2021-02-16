import PQueue from "p-queue";
import Bottle from "bottlejs";
import { SiVuexStore } from "@/store";
import _ from "lodash";

import { PartyBusEvent } from "./partyBus/partyBusEvent";

export class PartyBusSubscription {
  storePath: string;
  storeModuleName: string;
  storeInstanceName: string;
  action?: string;
  eventName: string;

  constructor(
    eventName: string,
    storeModuleName: string,
    storeInstanceName: string,
    action?: string,
  ) {
    this.eventName = eventName;
    this.storeModuleName = storeModuleName;
    this.storeInstanceName = storeInstanceName;
    this.storePath = `${this.storeModuleName}/${this.storeInstanceName}`;
    if (action) {
      this.action = action;
      this.storePath = `${this.storePath}/${this.action}`;
    } else {
      this.storePath = `${this.storePath}/on${this.eventName}`;
    }
  }
}

export class PartyBus {
  pq: PQueue;
  subscriptions: {
    [eventName: string]: Set<PartyBusSubscription>;
  };

  constructor() {
    this.pq = new PQueue();
    this.subscriptions = {};
  }

  async sendToSubscribers(event: PartyBusEvent) {
    if (this.subscriptions[event.eventName]) {
      let bottle = Bottle.pop("default");
      let store: SiVuexStore = bottle.container.Store;
      for (const subscription of this.subscriptions[event.eventName].values()) {
        if (
          store.hasModule([
            subscription.storeModuleName,
            subscription.storeInstanceName,
          ])
        ) {
          await store.dispatch(subscription.storePath, event);
        } else {
          // The store has gone, so the subscription goes too
          this.subscriptions[event.eventName].delete(subscription);
        }
      }
    }
  }

  publish(event: PartyBusEvent) {
    this.pq
      .add(async () => this.sendToSubscribers(event))
      .then(
        () => {},
        e => console.log("Could not send event to subscribers", { e }),
      );
  }

  subscribeToEvents(
    storeModuleName: string,
    storeInstanceName: string,
    events: { eventName: () => string }[],
  ) {
    for (const event of events) {
      this.subscribe(
        new PartyBusSubscription(
          event.eventName(),
          storeModuleName,
          storeInstanceName,
        ),
      );
    }
  }

  subscribe(subscription: PartyBusSubscription) {
    if (this.subscriptions[subscription.eventName]) {
      this.subscriptions[subscription.eventName].add(subscription);
    } else {
      this.subscriptions[subscription.eventName] = new Set([]);
      this.subscriptions[subscription.eventName].add(subscription);
    }
  }
}
