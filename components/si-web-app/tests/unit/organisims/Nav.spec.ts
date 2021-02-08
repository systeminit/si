import { render, fireEvent, waitFor } from "@testing-library/vue";
import { storeData } from "@/store";
import _ from "lodash";
import routes from "@/router/routes";
import { bottleSetup, bottleClear, bottleSetStore } from "@/di";
import { createBillingAccountAndLogin } from "../../support";
import Bottle from "bottlejs";
import Component from "@/organisims/Nav.vue";

describe("Nav.vue", () => {
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

  describe("loading", () => {
    test("shows default workspace name", async () => {
      let { getByText, queryByTestId } = render(
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

      expect(getByText("default Workspace"));
    });
  });

  describe("logout", () => {
    test("button works and redirects", async () => {
      let { findByTestId, getByLabelText, queryByTestId } = render(
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

      let logoutButton = getByLabelText("Logout");
      fireEvent.click(logoutButton);

      expect(queryByTestId("error-message")).not.toBeInTheDocument();

      await waitFor(async () => {
        let location = await findByTestId("location-display-nav");
        expect(location).toHaveTextContent(RegExp("^/authenticate/login$"));
      });
    });
  });
});
