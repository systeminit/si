import { render, fireEvent, waitFor } from "@testing-library/vue";
import { storeData } from "@/store";
import _ from "lodash";
import routes from "@/router/routes";
import { bottleSetup, bottleClear, bottleSetStore } from "@/di";
import {
  createFakeName,
  createBillingAccountAndLogin,
  INewBillingAccount,
} from "../../support";
import Bottle from "bottlejs";

import Component from "@/wads/LoginWad.vue";

describe("LoginWad.vue", () => {
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
    let { debug, getByLabelText, queryByTestId, findByTestId } = render(
      Component,
      {
        routes,
        // @ts-ignore
        store: initialStoreData,
      },
      (_localVue, store, router) => {
        bottleClear();
        bottleSetStore(store, router);
        router.push("/authenticate/login");
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

    await waitFor(async () => {
      let location = await findByTestId("location-display");
      expect(location).toHaveTextContent(RegExp("^/$"));
    });
  });

  test("cannot login with a bad password", async () => {
    let { getByLabelText, findByTestId, findByLabelText } = render(
      Component,
      {
        routes,
        // @ts-ignore
        store: initialStoreData,
      },
      (_localVue, store, router) => {
        bottleClear();
        bottleSetStore(store, router);
        router.push("/authenticate/login");
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

    await waitFor(async () => {
      let location = await findByTestId("location-display");
      expect(location).toHaveTextContent("/authenticate/login");
    });
  });

  test("can decide to sign up", async () => {
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
        router.push("/authenticate/login");
      },
    );

    let signupButton = getByLabelText("Sign Up");
    await fireEvent.click(signupButton);

    expect(queryByTestId("error-message")).not.toBeInTheDocument();

    await waitFor(async () => {
      let location = await findByTestId("location-display");
      expect(location).toHaveTextContent("/authenticate/signup");
    });
  });
});
