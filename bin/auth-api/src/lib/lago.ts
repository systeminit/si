import { Client } from "lago-javascript-client";

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
