import { render, waitFor } from "@testing-library/vue";
import { storeData } from "@/store";
import _ from "lodash";
import routes from "@/router/routes";
import { bottleSetup, bottleClear, bottleSetStore } from "@/di";
import { createBillingAccountAndLogin } from "../../support";
import Bottle from "bottlejs";

import Component from "@/pages/Home.vue";

describe("Home.vue", () => {
  let initialStoreData: any;
  let storeState: string;

  beforeEach(async () => {
    bottleSetup(storeData);
    initialStoreData = _.cloneDeep(storeData);
    initialStoreData.state = { version: "42" };
    await createBillingAccountAndLogin();
    let bottle = Bottle.pop("default");
    let store = bottle.container.Store;
    // Get the user and billing account in the session
    await store.dispatch("session/isAuthenticated");
    storeState = JSON.stringify(store.state);
  });

  afterEach(async () => {
    bottleClear();
  });

  test("loads a default workspace and organization; then redirects", async () => {
    let { queryByTestId, queryAllByTestId } = render(
      Component,
      {
        routes,
        // @ts-ignore
        store: initialStoreData,
      },
      (_localVue, store, router) => {
        bottleClear();
        bottleSetStore(store, router);
        store.replaceState(JSON.parse(storeState));
        router.push({ name: "home" });
      },
    );

    expect(queryByTestId("error-message")).not.toBeInTheDocument();

    await waitFor(async () => {
      let location = queryAllByTestId("location-display-homepage");
      expect(location.pop()).toHaveTextContent(RegExp("/o/(.+)/w/(.+)/a"));
    });
  });
});
