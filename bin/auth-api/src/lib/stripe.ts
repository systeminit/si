import Stripe from "stripe";

const stripe = new Stripe(process.env.STRIPE_API_KEY as string);

export async function checkCustomerPaymentMethodSet(stripeCustomerId: string) {
  const resp = await stripe.customers.listPaymentMethods(stripeCustomerId);
  if (resp.data && resp.data.length > 0) {
    return true;
  }

  return false;
}
