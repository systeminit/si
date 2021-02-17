import { render, fireEvent, waitFor } from "@testing-library/vue";
import userEvent from "@testing-library/user-event";
import { storeData, InstanceStoreContext } from "@/store";
import { registerStatusBar, StatusBarStore } from "@/store/modules/statusBar";
import {
  registerApplicationContext,
  ApplicationContextStore,
} from "@/store/modules/applicationContext";
import _ from "lodash";
import routes from "@/router/routes";
import { bottleSetup, bottleClear, bottleSetStore } from "@/di";
import {
  createBillingAccountAndLogin,
  createApplication,
  INewBillingAccount,
  createChangeSet,
} from "../../support";
import Bottle from "bottlejs";
import Component from "@/organisims/ApplicationContext.vue";
import { Entity } from "@/api/sdf/model/entity";
import { ISetDefaultsReply } from "@/store/modules/session";
import { ICreateChangeSetAndEditSessionReplySuccess } from "@/api/sdf/dal/applicationContextDal";

interface Setup {
  initialStoreData: any;
  storeState: string;
  application: Entity;
  nba: INewBillingAccount;
  sessionDefaults: ISetDefaultsReply;
  applicationContextCtx: InstanceStoreContext<ApplicationContextStore>;
  statusBarCtx: InstanceStoreContext<StatusBarStore>;
  initialChangeSet: ICreateChangeSetAndEditSessionReplySuccess;
}

async function setup(): Promise<Setup> {
  bottleSetup(storeData);
  let storeState = "";
  let initialStoreData = _.cloneDeep(storeData);
  initialStoreData.state = { version: "42" };
  let nba = await createBillingAccountAndLogin();
  let bottle = Bottle.pop("default");
  let store = bottle.container.Store;
  // Get the user and billing account in the session
  await store.dispatch("session/isAuthenticated");
  // Get the default organization and workspace
  let sessionDefaults = await store.dispatch("session/setDefaults");
  let application = await createApplication();
  let applicationContextCtx: InstanceStoreContext<ApplicationContextStore> = new InstanceStoreContext(
    {
      storeName: "applicationContext",
      componentId: "ApplicationDetails",
      instanceId: "applicationDetails",
    },
  );
  let statusBarCtx: InstanceStoreContext<StatusBarStore> = new InstanceStoreContext(
    {
      storeName: "statusBar",
      componentId: "ApplicationDetails",
      instanceId: "applicationDetails",
    },
  );
  let reply = await createChangeSet();
  if (reply.error) {
    throw new Error(`cannot create new change set: ${reply.error.message}`);
  }
  let initialChangeSet = reply;
  storeState = JSON.stringify(store.state);
  return {
    nba,
    sessionDefaults,
    application,
    applicationContextCtx,
    statusBarCtx,
    initialStoreData,
    initialChangeSet,
    storeState,
  };
}

describe("ApplicationContext.vue", () => {
  afterEach(async () => {
    bottleClear();
  });

  describe("render", () => {
    test("starts in view mode with no change set selected", async () => {
      let {
        sessionDefaults,
        application,
        applicationContextCtx,
        storeState,
        statusBarCtx,
        initialStoreData,
      } = await setup();

      let {
        getByTestId,
        getByLabelText,
        queryByLabelText,
        findByText,
        findByTestId,
        findByRole,
      } = render(
        Component,
        {
          routes,
          // @ts-ignore
          store: initialStoreData,
          propsData: {
            workspaceId: sessionDefaults.workspace?.id,
            applicationId: application.id,
            applicationContextCtx,
          },
        },
        (_localVue, store, router) => {
          bottleClear();
          bottleSetStore(store, router);
          store.replaceState(JSON.parse(storeState));
          registerApplicationContext(applicationContextCtx, statusBarCtx);
          registerStatusBar(statusBarCtx.instanceId);
        },
      );

      // Application Name is on the page
      let appNameRegex = `applications\/${application.name}`;
      expect(await findByText(new RegExp(appNameRegex))).toBeInTheDocument();

      // The system select is in the page, and set correctly.
      let systemSelect = await findByTestId("systemSelect");
      expect(systemSelect).toHaveValue(sessionDefaults?.system?.id);

      // The edit button is in the dom
      expect(getByLabelText("edit")).toBeInTheDocument();

      // The execute button is in the dom, and disabled
      let executeButton = queryByLabelText("execute");
      expect(executeButton).toBeInTheDocument();
      expect(executeButton).toBeDisabled();
    });

    test("edit button allows for change set creation", async () => {
      let {
        sessionDefaults,
        application,
        applicationContextCtx,
        storeState,
        statusBarCtx,
        initialStoreData,
      } = await setup();

      let {
        getByTestId,
        getByLabelText,
        queryByLabelText,
        findByText,
        getByDisplayValue,
      } = render(
        Component,
        {
          routes,
          // @ts-ignore
          store: initialStoreData,
          propsData: {
            workspaceId: sessionDefaults.workspace?.id,
            applicationId: application.id,
            applicationContextCtx,
          },
        },
        (_localVue, store, router) => {
          bottleClear();
          bottleSetStore(store, router);
          store.replaceState(JSON.parse(storeState));
          registerApplicationContext(applicationContextCtx, statusBarCtx);
          registerStatusBar(statusBarCtx.instanceId);
        },
      );

      let editButton = getByLabelText("edit");
      await fireEvent.click(editButton);

      expect(
        await findByText(/a changeSet is required to make edits/i),
      ).toBeInTheDocument();

      let newChangeSetTextbox = getByTestId("new-change-set-name");
      fireEvent.update(newChangeSetTextbox, "patience");

      let createButton = getByLabelText("create");
      await fireEvent.click(createButton);

      await waitFor(() => {
        expect(queryByLabelText("edit")).not.toBeInTheDocument();
      });

      expect(getByLabelText("done")).toBeInTheDocument();
      expect(getByLabelText("cancel")).toBeInTheDocument();
      expect(getByLabelText("execute")).toBeInTheDocument();

      let selectCurrentChangeSet = getByDisplayValue("patience");
      expect(selectCurrentChangeSet).toBeDisabled();
    });

    test("edit button allows for change set selection", async () => {
      let {
        sessionDefaults,
        application,
        applicationContextCtx,
        storeState,
        statusBarCtx,
        initialChangeSet,
        initialStoreData,
      } = await setup();

      let {
        getByTestId,
        getByLabelText,
        queryByLabelText,
        debug,
        queryByTestId,
        findByText,
        getByDisplayValue,
        findByDisplayValue,
      } = render(
        Component,
        {
          routes,
          // @ts-ignore
          store: initialStoreData,
          propsData: {
            workspaceId: sessionDefaults.workspace?.id,
            applicationId: application.id,
            applicationContextCtx,
          },
        },
        (_localVue, store, router) => {
          bottleClear();
          bottleSetStore(store, router);
          store.replaceState(JSON.parse(storeState));
          registerApplicationContext(applicationContextCtx, statusBarCtx);
          registerStatusBar(statusBarCtx.instanceId);
        },
      );

      let editButton = getByLabelText("edit");
      await fireEvent.click(editButton);

      expect(
        await findByText(/a changeSet is required to make edits/i),
      ).toBeInTheDocument();

      await waitFor(() => {
        let selectChangeSetEdit = getByTestId("selectCurrentChangeSetEdit");
        userEvent.selectOptions(selectChangeSetEdit, [
          initialChangeSet.changeSet.id,
        ]);
      });

      await waitFor(() => {
        expect(queryByLabelText("edit")).not.toBeInTheDocument();
      });

      await waitFor(() => {
        expect(getByLabelText("done")).toBeInTheDocument();
        expect(getByLabelText("cancel")).toBeInTheDocument();
        expect(getByLabelText("execute")).toBeInTheDocument();
      });

      let selectCurrentChangeSet = getByDisplayValue(
        initialChangeSet.changeSet.name,
      );
      expect(selectCurrentChangeSet).toBeDisabled();
    });

    test("done button exits edit mode", async () => {
      let {
        sessionDefaults,
        application,
        applicationContextCtx,
        storeState,
        statusBarCtx,
        initialStoreData,
      } = await setup();

      let {
        getByTestId,
        getByLabelText,
        queryByLabelText,
        findByText,
        getByDisplayValue,
        queryByDisplayValue,
      } = render(
        Component,
        {
          routes,
          // @ts-ignore
          store: initialStoreData,
          propsData: {
            workspaceId: sessionDefaults.workspace?.id,
            applicationId: application.id,
            applicationContextCtx,
          },
        },
        (_localVue, store, router) => {
          bottleClear();
          bottleSetStore(store, router);
          store.replaceState(JSON.parse(storeState));
          registerApplicationContext(applicationContextCtx, statusBarCtx);
          registerStatusBar(statusBarCtx.instanceId);
        },
      );

      let editButton = getByLabelText("edit");
      await fireEvent.click(editButton);

      expect(
        await findByText(/a changeSet is required to make edits/i),
      ).toBeInTheDocument();

      let newChangeSetTextbox = getByTestId("new-change-set-name");
      fireEvent.update(newChangeSetTextbox, "patience");

      let createButton = getByLabelText("create");
      await fireEvent.click(createButton);

      await waitFor(() => {
        expect(queryByLabelText("edit")).not.toBeInTheDocument();
      });

      let doneButton = getByLabelText("done");
      fireEvent.click(doneButton);

      await waitFor(() => {
        let selectCurrentChangeSet = queryByDisplayValue("patience");
        expect(selectCurrentChangeSet).not.toBeDisabled();
      });

      expect(getByLabelText("edit")).toBeInTheDocument();
      expect(queryByLabelText("cancel")).not.toBeInTheDocument();
    });

    test("cancel button exits edit mode", async () => {
      let {
        sessionDefaults,
        application,
        applicationContextCtx,
        storeState,
        statusBarCtx,
        initialStoreData,
      } = await setup();

      let {
        getByTestId,
        getByLabelText,
        queryByLabelText,
        findByText,
        getByDisplayValue,
        queryByDisplayValue,
      } = render(
        Component,
        {
          routes,
          // @ts-ignore
          store: initialStoreData,
          propsData: {
            workspaceId: sessionDefaults.workspace?.id,
            applicationId: application.id,
            applicationContextCtx,
          },
        },
        (_localVue, store, router) => {
          bottleClear();
          bottleSetStore(store, router);
          store.replaceState(JSON.parse(storeState));
          registerApplicationContext(applicationContextCtx, statusBarCtx);
          registerStatusBar(statusBarCtx.instanceId);
        },
      );

      let editButton = getByLabelText("edit");
      await fireEvent.click(editButton);

      expect(
        await findByText(/a changeSet is required to make edits/i),
      ).toBeInTheDocument();

      let newChangeSetTextbox = getByTestId("new-change-set-name");
      fireEvent.update(newChangeSetTextbox, "patience");

      let createButton = getByLabelText("create");
      await fireEvent.click(createButton);

      await waitFor(() => {
        expect(queryByLabelText("edit")).not.toBeInTheDocument();
      });

      let cancelButton = getByLabelText("cancel");
      fireEvent.click(cancelButton);

      await waitFor(() => {
        let selectCurrentChangeSet = queryByDisplayValue("patience");
        expect(selectCurrentChangeSet).not.toBeDisabled();
      });

      expect(getByLabelText("edit")).toBeInTheDocument();
      expect(queryByLabelText("done")).not.toBeInTheDocument();
    });

    test("selecting a change set lets you go to edit mode", async () => {
      let {
        sessionDefaults,
        application,
        applicationContextCtx,
        storeState,
        statusBarCtx,
        initialStoreData,
        initialChangeSet,
      } = await setup();

      let {
        getByTestId,
        getByLabelText,
        queryByLabelText,
        findByText,
        findByLabelText,
        getByDisplayValue,
        queryByDisplayValue,
        queryByText,
        debug,
      } = render(
        Component,
        {
          routes,
          // @ts-ignore
          store: initialStoreData,
          propsData: {
            workspaceId: sessionDefaults.workspace?.id,
            applicationId: application.id,
            applicationContextCtx,
          },
        },
        (_localVue, store, router) => {
          bottleClear();
          bottleSetStore(store, router);
          store.replaceState(JSON.parse(storeState));
          registerApplicationContext(applicationContextCtx, statusBarCtx);
          registerStatusBar(statusBarCtx.instanceId);
        },
      );

      waitFor(() => {
        let selectChangeSet = getByTestId("selectCurrentChangeSet");
        userEvent.selectOptions(selectChangeSet, [
          initialChangeSet.changeSet.name,
        ]);
      });
    });
  });
});
