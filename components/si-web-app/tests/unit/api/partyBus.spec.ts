import Bottle from "bottlejs";
import { SiVuexStore, storeData } from "@/store";
import { Module } from "vuex";
import { CurrentChangeSetEvent } from "@/api/partyBus/currentChangeSetEvent";
import { PartyBus } from "@/api/partyBus";
import { bottleSetup, bottleClear } from "@/di";
import { ChangeSet, ChangeSetStatus } from "@/api/sdf/model/changeSet";
import { waitFor } from "@testing-library/vue";

export const testStore: Module<any, any> = {
  namespaced: true,
  state(): any {
    return {
      changeSetName: null,
    };
  },
  mutations: {
    setChangeSetName(state, payload: string | null) {
      state.changeSetName = payload;
    },
  },
  actions: {
    onCurrentChangeSet({ commit }, event: CurrentChangeSetEvent) {
      commit("setChangeSetName", event.changeSet?.name);
    },
  },
};

describe("PartyBus", () => {
  beforeEach(() => {
    bottleSetup(storeData);
    let bottle = Bottle.pop("default");
    let store: SiVuexStore = bottle.container.Store;
    store.registerModule("testStore", { namespaced: true });
    store.registerModule(["testStore", "coolInstance"], testStore);
    store.registerModule(["testStore", "neatInstance"], testStore);
  });

  afterEach(() => {
    bottleClear();
  });

  test("subscribe to events", async () => {
    let bottle = Bottle.pop("default");
    let partyBus: PartyBus = bottle.container.PartyBus;
    partyBus.subscribeToEvents("testStore", "coolInstance", [
      CurrentChangeSetEvent,
    ]);
    expect(partyBus.subscriptions).toHaveProperty("CurrentChangeSet");
    let subscription = partyBus.subscriptions["CurrentChangeSet"]
      .values()
      .next().value;
    expect(subscription.storeModuleName).toBe("testStore");
    expect(subscription.storeInstanceName).toBe("coolInstance");
    expect(subscription.storePath).toBe(
      "testStore/coolInstance/onCurrentChangeSet",
    );
  });

  test("publish an event", async () => {
    let bottle = Bottle.pop("default");
    let store: SiVuexStore = bottle.container.Store;
    let partyBus: PartyBus = bottle.container.PartyBus;
    partyBus.subscribeToEvents("testStore", "coolInstance", [
      CurrentChangeSetEvent,
    ]);
    let fakeChangeSet = new ChangeSet({
      id: "changeSet:1",
      name: "tom nook",
      note: "is a landlord",
      status: ChangeSetStatus.Open,
      siStorable: {
        typeName: "changeSet",
        objectId: "changeSet:1",
        billingAccountId: "ba:1",
        organizationId: "o:1",
        workspaceId: "w:1",
        tenantIds: [],
        deleted: false,
      },
    });
    new CurrentChangeSetEvent(fakeChangeSet).publish();
    waitFor(() => {
      // @ts-ignore
      expect(store.state.testStore.coolInstance.changeSetName).toBe("tom nook");
    });
  });

  test("publish an event to multiple subscribers", async () => {
    let bottle = Bottle.pop("default");
    let store: SiVuexStore = bottle.container.Store;
    let partyBus: PartyBus = bottle.container.PartyBus;
    partyBus.subscribeToEvents("testStore", "coolInstance", [
      CurrentChangeSetEvent,
    ]);
    partyBus.subscribeToEvents("testStore", "neatInstance", [
      CurrentChangeSetEvent,
    ]);
    let fakeChangeSet = new ChangeSet({
      id: "changeSet:1",
      name: "tom nook",
      note: "is a landlord",
      status: ChangeSetStatus.Open,
      siStorable: {
        typeName: "changeSet",
        objectId: "changeSet:1",
        billingAccountId: "ba:1",
        organizationId: "o:1",
        workspaceId: "w:1",
        tenantIds: [],
        deleted: false,
      },
    });
    new CurrentChangeSetEvent(fakeChangeSet).publish();
    waitFor(() => {
      // @ts-ignore
      expect(store.state.testStore.coolInstance.changeSetName).toBe("tom nook");
      // @ts-ignore
      expect(store.state.testStore.neatInstance.changeSetName).toBe("tom nook");
    });
  });

  test("unsubscribe automatically on publish to non-existent subscriber", async () => {
    let bottle = Bottle.pop("default");
    let store: SiVuexStore = bottle.container.Store;
    let partyBus: PartyBus = bottle.container.PartyBus;
    partyBus.subscribeToEvents("testStore", "coolInstance", [
      CurrentChangeSetEvent,
    ]);
    let fakeChangeSet = new ChangeSet({
      id: "changeSet:1",
      name: "tom nook",
      note: "is a landlord",
      status: ChangeSetStatus.Open,
      siStorable: {
        typeName: "changeSet",
        objectId: "changeSet:1",
        billingAccountId: "ba:1",
        organizationId: "o:1",
        workspaceId: "w:1",
        tenantIds: [],
        deleted: false,
      },
    });
    new CurrentChangeSetEvent(fakeChangeSet).publish();
    waitFor(() => {
      // @ts-ignore
      expect(store.state.testStore.coolInstance.changeSetName).toBe("tom nook");
    });
    store.unregisterModule(["testStore", "coolInstance"]);
    new CurrentChangeSetEvent(fakeChangeSet).publish();
    expect(partyBus.subscriptions["CurrentChangeSet"].size).toBe(0);
  });
});
