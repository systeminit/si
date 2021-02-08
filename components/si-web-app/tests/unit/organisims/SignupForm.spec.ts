import { render, fireEvent, waitFor } from "@testing-library/vue";
import { storeData } from "@/store";
import _ from "lodash";
import routes from "@/router/routes";
import { bottleSetup, bottleClear, bottleSetStore } from "@/di";
import { createFakeName } from "../../support";

import Component from "@/organisims/SignupForm.vue";

describe("SignupForm.vue", () => {
  let initialStoreData: any;

  beforeEach(async () => {
    bottleSetup(storeData);
    initialStoreData = _.cloneDeep(storeData);
    initialStoreData.state = { version: "42" };
  });

  afterEach(async () => {
    bottleClear();
  });

  test("can create a billing account", async () => {
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

    let billingAccountName = createFakeName();

    let billingAccountNameInput = getByLabelText("Billing Account Name:");
    await fireEvent.update(billingAccountNameInput, billingAccountName);

    let billingAccountDescriptionInput = getByLabelText(
      "Billing Account Description:",
    );
    await fireEvent.update(billingAccountDescriptionInput, "acme");

    let userFullNameInput = getByLabelText("User Full Name:");
    await fireEvent.update(userFullNameInput, "a");

    let userEmailInput = getByLabelText("User E-Mail:");
    await fireEvent.update(userEmailInput, "a");

    let userPasswordInput = getByLabelText("User Password:");
    await fireEvent.update(userPasswordInput, "a");

    let userPasswordSecondInput = getByLabelText("User Password Again:");
    await fireEvent.update(userPasswordSecondInput, "a");

    let signupButton = getByLabelText("Sign Up");
    await fireEvent.click(signupButton);
  });

  test("can cancel back to login", async () => {
    let { getByLabelText } = render(
      Component,
      {
        routes,
        // @ts-ignore
        store: initialStoreData,
      },
      (_localVue, store, router) => {
        bottleClear();
        bottleSetStore(store, router);
        router.push("/authenticate/signup");
      },
    );

    let cancelButton = getByLabelText("Cancel");
    await fireEvent.click(cancelButton);
  });
});
