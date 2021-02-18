import { render, fireEvent } from "@testing-library/vue";
import { storeData } from "@/store";
import _ from "lodash";
import routes from "@/router/routes";
import { bottleSetup, bottleClear, bottleSetStore } from "@/di";

import Component from "@/templates/Secret.vue";

describe("Secret.vue", () => {
  let initialStoreData: any;

  beforeEach(async () => {
    bottleSetup(storeData);
    initialStoreData = _.cloneDeep(storeData);
    initialStoreData.state = { version: "42" };
  });

  afterEach(async () => {
    bottleClear();
  });

  test("the new secret button shows the create modal", async () => {
    let { getByLabelText, queryByText } = render(
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

    expect(queryByText("Create new secret")).not.toBeInTheDocument();

    let newSecretButton = getByLabelText("New Secret");
    await fireEvent.click(newSecretButton);

    expect(queryByText("Create new secret")).toBeInTheDocument();
  });

  test("the create modal can be destroyed", async () => {
    let { getByLabelText, queryByText, getByRole } = render(
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

    let newSecretButton = getByLabelText("New Secret");
    await fireEvent.click(newSecretButton);

    let closeButton = getByRole("button", {
      name: "Close New Secret Modal",
    });
    await fireEvent.click(closeButton);

    expect(queryByText("Create new secret")).not.toBeInTheDocument();
  });
});
