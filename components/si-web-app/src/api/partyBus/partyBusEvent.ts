import Bottle from "bottlejs";

export abstract class PartyBusEvent {
  eventName: string;

  constructor(eventName: PartyBusEvent["eventName"]) {
    this.eventName = eventName;
  }

  publish() {
    let bottle = Bottle.pop("default");
    let partyBus = bottle.container.PartyBus;
    partyBus.publish(this);
  }

  static eventName(): string {
    throw new Error(
      "You must provide a static eventName in your implementation for a PartyBusEvent! Bug!",
    );
  }
}
