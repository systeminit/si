import { render, fireEvent, waitFor } from "@testing-library/vue";
import { storeData } from "@/store";
import _ from "lodash";
import routes from "@/router/routes";
import { bottleSetup, bottleClear, bottleSetStore } from "@/di";
import {
  createBillingAccountAndLogin,
  INewBillingAccount,
} from "../../support";

import Component from "@/organisims/LoginForm.vue";

describe("LoginForm.vue", () => {
  let initialStoreData: any;
  let nba: INewBillingAccount;

  beforeEach(async () => {
    bottleSetup(storeData);
    initialStoreData = _.cloneDeep(storeData);
    initialStoreData.state = { version: "42" };
    nba = await createBillingAccountAndLogin();
  });

  afterEach(async () => {
    bottleClear();
  });

  test("can login", async () => {
    let { getByLabelText, queryByTestId, findByTestId } = render(
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

    let billingAccountNameInput = getByLabelText("Billing Account Name:");
    await fireEvent.update(billingAccountNameInput, nba.billingAccount.name);

    let userEmailInput = getByLabelText("User E-Mail:");
    await fireEvent.update(userEmailInput, nba.user.email);

    let userPasswordInput = getByLabelText("User Password:");
    await fireEvent.update(userPasswordInput, "a");

    let loginButton = getByLabelText("Login");
    await fireEvent.click(loginButton);

    expect(queryByTestId("error-message")).not.toBeInTheDocument();
  });

  test("cannot login with a bad password", async () => {
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
      },
    );

    let billingAccountNameInput = getByLabelText("Billing Account Name:");
    await fireEvent.update(billingAccountNameInput, nba.billingAccount.name);

    let userEmailInput = getByLabelText("User E-Mail:");
    await fireEvent.update(userEmailInput, nba.user.email);

    let userPasswordInput = getByLabelText("User Password:");
    await fireEvent.update(userPasswordInput, "c");

    let loginButton = getByLabelText("Login");
    await fireEvent.click(loginButton);

    let errorMessage = await findByTestId("error-message");
    expect(errorMessage).toHaveTextContent("Login error; please try again!");
  });

  test("can decide to sign up", async () => {
    let { getByLabelText, queryByTestId } = render(
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

    let signupButton = getByLabelText("Sign Up");
    await fireEvent.click(signupButton);

    expect(queryByTestId("error-message")).not.toBeInTheDocument();
  });
});
