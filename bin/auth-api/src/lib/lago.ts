import { Client, Country } from "lago-javascript-client";

const client = Client(process.env.LAGO_API_KEY as string, {
  baseUrl: "https://api.getlago.com/api/v1",
});

export async function createCustomer(
  userPk: string,
  firstName: string,
  lastName: string,
  email: string,
) {
  return client.customers.createCustomer({
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
  });
}

export async function createTrialSubscription(userPk: string) {
  const plan_code = "launch_trial";
  const external_id = `${userPk}_${plan_code}`;
  const trialEndDate = new Date(
    new Date().getTime() + 30 * 24 * 60 * 60 * 1000,
  );
  trialEndDate.setUTCHours(0, 0, 0, 0);
  return client.subscriptions.createSubscription({
    subscription: {
      external_id,
      external_customer_id: userPk,
      ending_at: `${trialEndDate.toISOString().split(".")[0]}Z`,
      plan_code,
    },
  });
}

export async function createPaidSubscription(userPk: string) {
  const plan_code = "launch_pay_as_you_go";
  const external_id = `${userPk}_${plan_code}`;
  const subscriptionStartDate = new Date(
    new Date().getTime() + 31 * 24 * 60 * 60 * 1000,
  );
  subscriptionStartDate.setUTCHours(0, 0, 0, 0);
  return client.subscriptions.createSubscription({
    subscription: {
      external_id,
      external_customer_id: userPk,
      subscription_at: subscriptionStartDate.toISOString(),
      plan_code,
    },
  });
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
  return client.customers.createCustomer({
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
  });
}

export async function getCustomerBillingDetails(userPk: string) {
  const resp = await client.customers.findCustomer(userPk);
  if (resp.ok) {
    return resp.data.customer;
  }

  return null;
}

export async function generateCustomerCheckoutUrl(userPk: string) {
  const resp = await client.customers.generateCustomerCheckoutUrl(userPk);
  if (resp.ok) {
    return resp.data.customer?.checkout_url;
  }

  return null;
}

export async function getCustomerPortalUrl(userPk: string) {
  const resp = await client.customers.getCustomerPortalUrl(userPk);
  if (resp.ok) {
    return resp.data.customer?.portal_url;
  }
  return null;
}
