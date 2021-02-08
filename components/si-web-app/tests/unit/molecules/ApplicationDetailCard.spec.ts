import { render } from "@testing-library/vue";
import { storeData } from "@/store";
import _ from "lodash";
import routes from "@/router/routes";
import { bottleSetup, bottleClear, bottleSetStore } from "@/di";
import {
  createBillingAccountAndLogin,
  createApplicationListEntry,
  setSessionDefaults,
  INewBillingAccount,
} from "../../support";
import Bottle from "bottlejs";

import Component from "@/molecules/ApplicationDetailCard.vue";
import { IApplicationListEntry } from "@/store/modules/application";
import { ISetDefaultsReply } from "@/store/modules/session";

describe("ApplicatonDetailCard.vue", () => {
  let initialStoreData: any;
  let defaults: ISetDefaultsReply;
  let appEntry: IApplicationListEntry;

  beforeEach(async () => {
    bottleSetup(storeData);
    initialStoreData = _.cloneDeep(storeData);
    initialStoreData.state = { version: "42" };
    await createBillingAccountAndLogin();
    let bottle = Bottle.pop("default");
    let store = bottle.container.Store;
    // Get the user and billing account in the session
    await store.dispatch("session/isAuthenticated");
    // Get the default organization and workspace
    defaults = await store.dispatch("session/setDefaults");
    appEntry = await createApplicationListEntry();
  });

  afterEach(async () => {
    bottleClear();
  });

  test("required data appears in the card", async () => {
    let { findByText, findByTestId } = render(
      Component,
      {
        routes,
        // @ts-ignore
        store: initialStoreData,
        propsData: {
          applicationEntry: appEntry,
          linkTo: {
            name: "applicationEditor",
            params: {
              workspaceId: defaults.workspace?.id,
              organizationId: defaults.organization?.id,
              applicationId: appEntry.application.id,
            },
          },
        },
      },
      (_localVue, store, router) => {
        bottleClear();
        bottleSetStore(store, router);
      },
    );

    expect(await findByText(appEntry.application.name)).toBeInTheDocument();
    expect(await findByTestId("changeSetCountsOpen")).toHaveTextContent("0");
    expect(await findByTestId("changeSetCountsClosed")).toHaveTextContent("1");
  });
});
