/* eslint-disable no-console */
import { ChargeObject, Client, Country } from "lago-javascript-client";
import _ from "lodash";

const client = Client(process.env.LAGO_API_KEY as string, {
  baseUrl: "https://api.getlago.com/api/v1",
});

// Helper to wrap Lago calls with timing
async function timedLagoCall<T>(operation: string, fn: () => Promise<T>): Promise<T> {
  const start = Date.now();
  try {
    const result = await fn();
    const duration_ms = Date.now() - start;
    console.log(JSON.stringify({
      timestamp: new Date().toISOString(),
      level: "info",
      type: "lago",
      operation,
      duration_ms,
      ...(duration_ms > 5000 && { slowCall: true }),
    }));
    return result;
  } catch (error) {
    const duration_ms = Date.now() - start;
    console.log(JSON.stringify({
      timestamp: new Date().toISOString(),
      level: "error",
      type: "lago",
      operation,
      duration_ms,
      ...(duration_ms > 5000 && { slowCall: true }),
      error: error instanceof Error ? error.message : String(error),
    }));
    throw error;
  }
}

export async function createCustomer(
  userPk: string,
  firstName: string,
  lastName: string,
  email: string,
) {
  return timedLagoCall("createCustomer", () => client.customers.createCustomer({
    customer: {
      external_id: userPk,
      name: `${firstName} ${lastName}`,
      email,
      billing_configuration: {
        payment_provider: "stripe",
        payment_provider_code: "stripe",
        sync: true,
        sync_with_provider: true,
        provider_payment_methods: ["card", "us_bank_account"],
      },
    },
  }));
}

export async function createTrialSubscription(userPk: string) {
  const plan_code = "launch_trial";
  const external_id = `${userPk}_${plan_code}`;
  const trialEndDate = new Date(
    new Date().getTime() + 30 * 24 * 60 * 60 * 1000,
  );
  trialEndDate.setUTCHours(0, 0, 0, 0);
  return timedLagoCall("createTrialSubscription", () => client.subscriptions.createSubscription({
    subscription: {
      external_id,
      external_customer_id: userPk,
      ending_at: `${trialEndDate.toISOString().split(".")[0]}Z`,
      plan_code,
    },
  }));
}

export async function createPaidSubscription(userPk: string) {
  const plan_code = "launch_pay_as_you_go";
  const external_id = `${userPk}_${plan_code}`;
  const subscriptionStartDate = new Date(
    new Date().getTime() + 31 * 24 * 60 * 60 * 1000,
  );
  subscriptionStartDate.setUTCHours(0, 0, 0, 0);
  return timedLagoCall("createPaidSubscription", () => client.subscriptions.createSubscription({
    subscription: {
      external_id,
      external_customer_id: userPk,
      subscription_at: subscriptionStartDate.toISOString(),
      plan_code,
    },
  }));
}

export type CustomerDetail = {
  id: string;
  firstName?: string | null;
  lastName?: string | null;
  email: string;
  companyInformation: {
    legalName?: string | null;
    legalNumber?: string | null;
    taxIdentificationNumber?: string | null;
    phoneNumber?: string | null;
  };
  billingInformation: {
    addressLine1?: string | null;
    addressLine2?: string | null;
    zipCode?: string | null;
    city?: string | null;
    state?: string | null;
    country?: string | null;
  };
  customerCheckoutUrl: string;
  customerPortalUrl: string;
};

export async function updateCustomerDetails(customer: CustomerDetail) {
  return timedLagoCall("updateCustomerDetails", () => client.customers.createCustomer({
    customer: {
      external_id: customer.id,
      name: `${customer.firstName} ${customer.lastName}`,
      email: customer.email,
      legal_name: customer.companyInformation.legalName,
      legal_number: customer.companyInformation.legalNumber,
      tax_identification_number:
        customer.companyInformation.taxIdentificationNumber,
      phone: customer.companyInformation.phoneNumber,
      address_line1: customer.billingInformation.addressLine1,
      address_line2: customer.billingInformation.addressLine2,
      city: customer.billingInformation.city,
      state: customer.billingInformation.state,
      zipcode: customer.billingInformation.zipCode,
      country: customer.billingInformation.country as Country,
      billing_configuration: {
        payment_provider: "stripe",
        payment_provider_code: "stripe",
        sync: true,
        sync_with_provider: true,
        provider_payment_methods: ["card", "us_bank_account"],
      },
    },
  }));
}

export async function getCustomerBillingDetails(userPk: string) {
  const resp = await timedLagoCall("getCustomerBillingDetails", () => client.customers.findCustomer(userPk));
  if (resp.ok) {
    return resp.data.customer;
  }

  return null;
}

export async function generateCustomerCheckoutUrl(userPk: string) {
  const resp = await timedLagoCall("generateCustomerCheckoutUrl", () => client.customers.generateCustomerCheckoutUrl(userPk));
  if (resp.ok) {
    return resp.data.customer?.checkout_url;
  }

  return null;
}

export async function getCustomerPortalUrl(userPk: string) {
  const resp = await timedLagoCall("getCustomerPortalUrl", () => client.customers.getCustomerPortalUrl(userPk));
  if (resp.ok) {
    return resp.data.customer?.portal_url;
  }
  return null;
}

export async function getCustomerActiveSubscription(userPk: string) {
  try {
    const trial_resp = await timedLagoCall("findSubscription_trial", () => client.subscriptions.findSubscription(
      `${userPk}_launch_trial`,
    ));
    if (trial_resp.ok && trial_resp.data.subscription.status === "active") {
      return {
        planCode: trial_resp.data.subscription.plan_code,
        subscriptionAt: trial_resp.data.subscription.subscription_at,
        endingAt: trial_resp.data.subscription.ending_at,
        isTrial: true,
        exceededFreeTier: false,
      };
    }
  } catch (err) {
    /* empty */
    // We default to an NOT_FOUND plan so we are fine here for now
  }

  try {
    const payg_resp = await timedLagoCall("findSubscription_payg", () => client.subscriptions.findSubscription(
      `${userPk}_launch_pay_as_you_go`,
    ));
    if (payg_resp.ok) {
      const charges = _.get(
        payg_resp.data.subscription,
        "plan.charges",
        [],
      ) as ChargeObject[];
      const paidCharge = _.find(
        charges,
        (charge: ChargeObject) => charge.billable_metric_code === "resource-hours" && parseFloat(_.get(charge, "properties.amount", "0")) > 0,
      );
      return {
        planCode: payg_resp.data.subscription.plan_code,
        subscriptionAt: payg_resp.data.subscription.subscription_at,
        endingAt: payg_resp.data.subscription.ending_at,
        isTrial: false,
        exceededFreeTier: !!paidCharge,
      };
    }
  } catch (err) {
    /* empty */
    // We default to an NOT_FOUND plan so we are fine here for now
  }

  try {
    const si_internal_resp = await timedLagoCall("findSubscription_internal", () => client.subscriptions.findSubscription(
      `${userPk}_si_internal`,
    ));
    if (si_internal_resp.ok) {
      const charges = _.get(
        si_internal_resp.data.subscription,
        "plan.charges",
        [],
      ) as ChargeObject[];
      const paidCharge = _.find(
        charges,
        (charge: ChargeObject) => charge.billable_metric_code === "resource-hours" && parseFloat(_.get(charge, "properties.amount", "0")) > 0,
      );
      return {
        planCode: si_internal_resp.data.subscription.plan_code,
        subscriptionAt: si_internal_resp.data.subscription.subscription_at,
        endingAt: si_internal_resp.data.subscription.ending_at,
        isTrial: false,
        exceededFreeTier: !!paidCharge,
      };
    }
  } catch (err) {
    /* empty */
    // We default to an NOT_FOUND plan so we are fine here for now
  }

  return {
    planCode: "NOT_FOUND",
    isTrial: false,
    exceededFreeTier: false,
  };
}
