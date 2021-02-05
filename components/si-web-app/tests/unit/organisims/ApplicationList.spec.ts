import { render, fireEvent, waitFor } from "@testing-library/vue";
import { storeData } from "@/store";
import _ from "lodash";
import routes from "@/router/routes";
import { bottleSetup, bottleClear, bottleSetStore } from "@/di";
import {
  createFakeName,
  createBillingAccountAndLogin,
  createApplication,
  setSessionDefaults,
} from "../../support";
import Bottle from "bottlejs";

import Component from "@/organisims/ApplicationList.vue";

describe("ApplicatonList.vue", () => {
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

  test("application list is empty", async () => {
    let { getByText } = render(
      Component,
      {
        routes,
        // @ts-ignore
        store: initialStoreData,
      },
      (_localVue, store, router) => {
        bottleClear();
        bottleSetStore(store, router);
        bottleClear();
        bottleSetStore(store, router);
        store.replaceState(JSON.parse(storeState));
      },
    );
    expect(getByText("No applications created yet!")).toBeInTheDocument();
  });

  test("created applications appear in the list", async () => {
    let { findByText } = render(
      Component,
      {
        routes,
        // @ts-ignore
        store: initialStoreData,
      },
      (_localVue, store, router) => {
        bottleClear();
        bottleSetStore(store, router);
      },
    );

    await setSessionDefaults();
    let app = await createApplication();
    expect(await findByText(app.name)).toBeInTheDocument();

    let app2 = await createApplication();
    expect(await findByText(app.name)).toBeInTheDocument();
    expect(await findByText(app2.name)).toBeInTheDocument();
  });
});
