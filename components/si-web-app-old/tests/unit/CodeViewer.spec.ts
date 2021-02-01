import { render, waitFor } from "@testing-library/vue";
import Component from "@/components/views/editor/EditorPropertyPanel/CodeViewer.vue";
import { storeData, SiVuexStore } from "@/store";
import _ from "lodash";
import routes from "@/router/routes";
import {
  createNewBillingAccount,
  createApplication,
  selectApplication,
  createChangeSet,
  createNode,
  selectNode,
} from "../support/state";
import { Node, NodeKind } from "@/api/sdf/model/node";
import { bottleSetup, bottleClear, bottleSetStore } from "@/di";
import FDBFactory from "fake-indexeddb/lib/FDBFactory";
import Bottle from "bottlejs";

interface IStateObjects {
  serviceNode: Node;
  kubernetesDeploymentNode: Node;
}

async function setStoreState(store: SiVuexStore): Promise<IStateObjects> {
  await createNewBillingAccount(store);
  let application = await createApplication(store);
  await selectApplication(store, application);
  await createChangeSet(store);
  let serviceNode = await createNode(store, {
    kind: NodeKind.Entity,
    objectType: "service",
  });
  let kubernetesDeploymentNode = await createNode(store, {
    kind: NodeKind.Entity,
    objectType: "kubernetesDeployment",
  });
  return {
    serviceNode,
    kubernetesDeploymentNode,
  };
}

describe("CodeViewer.vue", () => {
  let initialStoreData: any;

  beforeEach(async () => {
    bottleSetup(storeData);
    initialStoreData = _.cloneDeep(storeData);
    initialStoreData.state = { version: "42" };
  });

  afterEach(async () => {
    indexedDB = new FDBFactory();
    bottleClear();
  });

  test("displays code properties correctly", async () => {
    let { findByText } = render(
      Component,
      {
        routes,
        // @ts-ignore
        store: initialStoreData,
      },
      (_localVue, store, _router) => {
        bottleClear();
        bottleSetStore(store);
      },
    );
    let bottle = Bottle.pop("default");
    let store = bottle.container.Store;

    const objs = await setStoreState(store);

    await selectNode(store, objs.serviceNode);

    waitFor(async () => {
      await findByText("# No Code!");
    });

    await selectNode(store, objs.kubernetesDeploymentNode);

    waitFor(
      async () => {
        await findByText("apiVersion: apps/v1");
        await findByText("kind: Deployment");
      },
      { timeout: 2000 },
    );

    await selectNode(store, objs.serviceNode);

    waitFor(async () => {
      await findByText("# No Code!");
    });
  });
});
