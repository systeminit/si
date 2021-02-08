import { render, fireEvent, waitFor } from "@testing-library/vue";
import { storeData } from "@/store";
import _ from "lodash";
import routes from "@/router/routes";
import { bottleSetup, bottleClear, bottleSetStore } from "@/di";
import { createFakeName, createBillingAccountAndLogin } from "../../support";
import Bottle from "bottlejs";

import Component from "@/organisims/ApplicationCreate.vue";

describe("ApplicatonCreate.vue", () => {
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

  test("can be submitted", async () => {
    let { getByLabelText, findByTestId } = render(
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

    let applicationName = createFakeName();

    let applicationNameInput = getByLabelText("Application Name:");
    await fireEvent.update(applicationNameInput, applicationName);

    let createButton = getByLabelText("Create");
    await fireEvent.click(createButton);

    await findByTestId("application-create-error-message-okay");
  });

  test("cancel clears the application name", async () => {
    let { getByLabelText, queryByLabelText, queryByTestId } = render(
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

    let applicationName = createFakeName();

    let applicationNameInput = getByLabelText("Application Name:");
    await fireEvent.update(applicationNameInput, applicationName);

    await waitFor(async () => {
      let currentInput = queryByLabelText("Application Name:");
      // @ts-ignore
      expect(currentInput.value).toEqual(applicationName);
    });

    let cancelButton = getByLabelText("Cancel");
    await fireEvent.click(cancelButton);

    await waitFor(async () => {
      let currentInput = queryByLabelText("Application Name:");
      // @ts-ignore
      expect(currentInput.value).toEqual("");
    });

    expect(queryByTestId("error-message")).not.toBeInTheDocument();
  });
});
