import { render, waitFor } from "@testing-library/vue";
import { storeData, SiVuexStore } from "@/store";
import _ from "lodash";
import routes from "@/router/routes";
import {
  createNewBillingAccount,
  createApplication,
  selectApplication,
  createChangeSet,
  selectChangeSet,
  createNode,
  selectNode,
  login,
} from "../support/state";
import { Node, NodeKind } from "@/api/sdf/model/node";
import { bottleSetup, bottleClear, bottleSetStore } from "@/di";
import FDBFactory from "fake-indexeddb/lib/FDBFactory";
import Bottle from "bottlejs";

import Component from "@/components/views/application/ApplicationDetails.vue";
import { BillingAccount } from "@/api/sdf/model/billingAccount";
import { User } from "@/api/sdf/model/user";
import { Organization } from "@/api/sdf/model/organization";
import { Workspace } from "@/api/sdf/model/workspace";
import { System } from "@/api/sdf/model/system";
import { Entity } from "@/api/sdf/model/entity";
import { ChangeSet } from "@/api/sdf/model/changeSet";

interface IStateObjects {
  nodes: {
    service: Node;
    kubernetesDeployment: Node;
  };
  changeSet: ChangeSet;
  billingAccount: BillingAccount;
  user: User;
  group: Record<string, any>;
  organization: Organization;
  workspace: Workspace;
  system: System;
  application: Entity;
}

async function setStoreState(store: SiVuexStore): Promise<IStateObjects> {
  let nba = await createNewBillingAccount(store);
  let application = await createApplication(store);
  await selectApplication(store, application);
  let changeSet = await createChangeSet(store);
  let serviceNode = await createNode(store, {
    kind: NodeKind.Entity,
    objectType: "service",
  });
  let kubernetesDeploymentNode = await createNode(store, {
    kind: NodeKind.Entity,
    objectType: "kubernetesDeployment",
  });
  // @ts-ignore
  return {
    ...nba,
    changeSet,
    application,
    nodes: {
      service: serviceNode,
      kubernetesDeployment: kubernetesDeploymentNode,
    },
  };
}

async function setEditorState(store: SiVuexStore, objs: IStateObjects) {
  await login(store, objs);
  await selectApplication(store, objs.application);
  await selectChangeSet(store, { id: objs.changeSet.id });
}

describe("ApplicationDetails.vue", () => {
  let initialStoreData: any;
  let initialObjs: IStateObjects;

  beforeEach(async () => {
    bottleSetup(storeData);
    initialStoreData = _.cloneDeep(storeData);
    initialStoreData.state = { version: "42" };
    let bottle = Bottle.pop("default");
    let store = bottle.container.Store;
    const objs = await setStoreState(store);
    initialObjs = objs;
  });

  afterEach(async () => {
    indexedDB = new FDBFactory();
    bottleClear();
  });

  test("can cancel a change set", async () => {
    const objs = initialObjs;
    let { findByText, debug, findByRole } = render(
      Component,
      {
        routes,
        // @ts-ignore
        store: initialStoreData,
        props: {
          organizationId: objs.organization.id,
          workspaceId: objs.workspace.id,
          applicationId: objs.application.id,
        },
      },
      (_localVue, store, _router) => {
        bottleClear();
        bottleSetStore(store);
      },
    );
    let bottle = Bottle.pop("default");
    let store = bottle.container.Store;
    await setEditorState(store, objs);
    await store.dispatch("loader/load");
    console.dir(store.state.editor);

    let editButton = await findByRole("button", { name: "edit" });

    //let bottle = Bottle.pop("default");
    //let store = bottle.container.Store;

    return;
  });
});
