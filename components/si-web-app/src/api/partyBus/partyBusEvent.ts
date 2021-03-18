import Bottle from "bottlejs";

export abstract class PartyBusEvent {
  eventName: string;
  sourcePanelId?: string;

  constructor(eventName: PartyBusEvent["eventName"], sourcePanelId?: string) {
    this.eventName = eventName;
    this.sourcePanelId = sourcePanelId;
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
