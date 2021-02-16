import { render, fireEvent, waitFor } from "@testing-library/vue";
import { storeData } from "@/store";
import _ from "lodash";
import routes from "@/router/routes";
import { bottleSetup, bottleClear, bottleSetStore } from "@/di";
import { createBillingAccountAndLogin } from "../../support";
import Bottle from "bottlejs";
import Component from "@/organisims/StatusBar.vue";
import { registerStatusBar } from "@/store/modules/statusBar";

describe("StatusBar.vue", () => {
  let initialStoreData: any;
  let storeState: string;

  beforeEach(async () => {
    bottleSetup(storeData);
    storeState = "";
    initialStoreData = _.cloneDeep(storeData);
    initialStoreData.state = { version: "42" };
    await createBillingAccountAndLogin();
    let bottle = Bottle.pop("default");
    let store = bottle.container.Store;
    // Get the user and billing account in the session
    await store.dispatch("session/isAuthenticated");
    // Get the default organization and workspace
    await store.dispatch("session/setDefaults");
    storeState = JSON.stringify(store.state);
  });

  afterEach(async () => {
    bottleClear();
  });

  describe("render", () => {
    test("displays the individual slices of status", async () => {
      let { getByTestId, findByText } = render(
        Component,
        {
          routes,
          // @ts-ignore
          store: initialStoreData,
          propsData: {
            instanceId: "testa",
          },
        },
        (_localVue, store, router) => {
          bottleClear();
          bottleSetStore(store, router);
          store.replaceState(JSON.parse(storeState));
          registerStatusBar("testa");
        },
      );

      expect(getByTestId("status-bar").children.length).toBe(0);

      let bottle = Bottle.pop("default");
      let store = bottle.container.Store;

      await store.dispatch(
        "statusBar/testa/setApplicationName",
        "fooFightersAreAtLeastOkay",
      );
      expect(await findByText(/fooFightersAreAtLeastOkay/)).toBeInTheDocument();

      await store.dispatch("statusBar/testa/setSystemName", "systemOfADown");
      expect(await findByText(/systemOfADown/)).toBeInTheDocument();

      await store.dispatch("statusBar/testa/setNodeName", "nodesAreThings");
      expect(await findByText(/nodesAreThings/)).toBeInTheDocument();

      await store.dispatch("statusBar/testa/setNodeType", "monadOfCourse");
      expect(await findByText(/monadOfCourse/)).toBeInTheDocument();

      await store.dispatch("statusBar/testa/setChangeSetName", "chchchanges");
      expect(await findByText(/chchchanges/)).toBeInTheDocument();

      await store.dispatch("statusBar/testa/setEditMode", false);
      expect(await findByText(/view/)).toBeInTheDocument();
    });
  });
});
