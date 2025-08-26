/**
 * Lago Subscription Migration Script
 *
 * This script migrates customers from their current "pay_as_you_go" subscriptions to a new
 * "launch_trial" subscription as part of the product launch transition.
 *
 * What it does:
 * 1. Fetches all customers from Lago billing platform
 * 2. For each customer with an active "pay_as_you_go" subscription:
 *    - Updates the existing subscription to end on 2025-08-27
 *    - Creates a new "launch_trial" subscription starting 2025-08-27, ending 2025-09-26
 * 3. Skips customers who already have launch_trial subscriptions
 *
 * Environment Variables:
 * - LAGO_API_KEY: Required for actual operations (not needed for dry-run)
 * - LAGO_API_URL: Optional, defaults to https://api.getlago.com
 *
 * Usage:
 * - Dry run: `deno run --allow-net --allow-env lago-migration.ts --dry-run`
 * - Execute: `deno run --allow-net --allow-env lago-migration.ts`
 * - Batch mode: `deno run --allow-net --allow-env lago-migration.ts --batch=10`
 *
 * Safety Features:
 * - Dry-run mode to preview changes before execution
 * - Comprehensive logging of all API operations
 * - Error handling and validation
 * - Skips customers who already have launch_trial subscriptions
 */

// @ts-nocheck
import { Client } from "npm:lago-javascript-client";

// Check for dry-run mode
const isDryRun = Deno.args.includes("--dry-run");

// Parse batch size from command line args only
const getBatchSize = (): number => {
  const batchArg = Deno.args.find((arg) => arg.startsWith("--batch="));
  if (batchArg) {
    const size = parseInt(batchArg.split("=")[1]);
    return isNaN(size) || size <= 0 ? Infinity : size;
  }

  return Infinity; // Default: process all eligible migrations
};

const batchSize = getBatchSize();

// Initialize Lago client
const apiKey = Deno.env.get("LAGO_API_KEY");
const apiUrl = Deno.env.get("LAGO_API_URL") || "https://api.getlago.com";

if (!apiKey && !isDryRun) {
  console.error("Error: LAGO_API_KEY environment variable is required (unless running in dry-run mode)");
  Deno.exit(1);
}

const lago = apiKey ? Client(apiKey, apiUrl) : null;

async function migrateSubscriptions() {
  try {
    console.log(`Starting subscription migration${isDryRun ? " (DRY RUN MODE)" : ""}...`);
    if (isDryRun) {
      console.log("🔍 DRY RUN: No actual changes will be made to subscriptions");
    }

    if (batchSize !== Infinity) {
      console.log(`📦 BATCH MODE: Will process maximum ${batchSize} migrations (excluding skipped customers)`);
    }

    // Step 1: List all customers
    console.log("🔍 Fetching all customers...");
    const customers = await getAllCustomers();
    console.log(`Found ${customers.length} customers`);

    let processedCount = 0;
    let migratedCount = 0;
    let skippedCount = 0;
    let foundSubscriptions = [];

    // Step 2: Process each customer
    for (const customer of customers) {
      // Stop processing if we've reached the batch limit (excluding skips)
      if (migratedCount >= batchSize) {
        console.log(`🛑 Reached batch limit of ${batchSize} migrations. Stopping processing.`);
        const remainingCustomers = customers.length - processedCount;
        if (remainingCustomers > 0) {
          console.log(`⏭️  ${remainingCustomers} remaining customers will be processed in next batch`);
        }
        break;
      }
      processedCount++;
      console.log(`Processing customer ${processedCount}/${customers.length}: ${customer.external_id}`);

      // Check for active pay_as_you_go subscription
      const payAsYouGoSubId = `${customer.external_id}_launch_pay_as_you_go`;
      const activeSubscription = await findActiveSubscription(customer.external_id, payAsYouGoSubId);

      if (activeSubscription) {
        const subData = activeSubscription.data?.subscription || activeSubscription;
        const subscriptionId = subData.external_id || subData.subscription_id;
        const currentEndDate = subData.ending_at || subData.terminated_at || "No end date";

        console.log(`  ✅ Found active pay_as_you_go subscription: ${subscriptionId}`);

        // Check if customer already has a pending launch_trial subscription
        const existingLaunchTrial = await checkForPendingLaunchTrial(customer.external_id);

        if (existingLaunchTrial) {
          console.log(
            `  ⏩ SKIPPING: Customer already has ${existingLaunchTrial.status} launch_trial subscription: ${existingLaunchTrial.external_id}`,
          );
          skippedCount++;
          continue;
        }

        foundSubscriptions.push({
          customerId: customer.external_id,
          subscriptionId: subscriptionId,
          currentEndDate: currentEndDate,
          newEndDate: "2025-08-27",
          newTrialStart: "2025-08-27",
          newTrialEnd: "2025-09-26",
        });

        if (isDryRun) {
          console.log(`  🔍 DRY RUN: Would update subscription end date to 2025-08-27`);
          console.log(`  🔍 DRY RUN: Would create new launch_trial subscription (2025-08-27 to 2025-09-26)`);
        } else {
          // Step 3: Update existing subscription to end on 8/27
          const payAsYouGoEndDate = new Date("2025-08-27");
          payAsYouGoEndDate.setUTCHours(0, 0, 0, 0);
          const formattedEndDate = `${payAsYouGoEndDate.toISOString().split(".")[0]}Z`;
          await updateSubscriptionEndDate(subscriptionId, formattedEndDate);
          console.log(`  ✅ Updated subscription end date to 2025-08-27`);

          // Step 4: Create new launch_trial subscription
          await createLaunchTrialSubscription(customer.external_id);
          console.log(`  ✅ Created new launch_trial subscription`);
        }

        migratedCount++;
      } else {
        console.log(`  ❌ No active pay_as_you_go subscription found`);
      }
    }

    console.log(`\nMigration completed!`);
    console.log(`  Processed: ${processedCount} customers`);
    console.log(`  ${isDryRun ? "Found" : "Migrated"}: ${migratedCount} subscriptions`);
    console.log(`  Skipped: ${skippedCount} customers (already have launch_trial)`);

    // Print summary report for dry-run
    if (isDryRun && foundSubscriptions.length > 0) {
      console.log("\n📋 DRY RUN SUMMARY:");
      console.log("=".repeat(80));
      foundSubscriptions.forEach((sub, index) => {
        console.log(`${index + 1}. Customer: ${sub.customerId}`);
        console.log(`   Current subscription: ${sub.subscriptionId}`);
        console.log(`   Current end date: ${sub.currentEndDate}`);
        console.log(`   → Would change end date to: ${sub.newEndDate}`);
        console.log(`   → Would create launch_trial: ${sub.newTrialStart} to ${sub.newTrialEnd}`);
        console.log("");
      });
      console.log(`Total subscriptions to migrate: ${foundSubscriptions.length}`);
      if (skippedCount > 0) {
        console.log(`Total customers skipped: ${skippedCount} (already have launch_trial subscriptions)`);
      }
      console.log("\n🚀 To execute these changes, run without --dry-run flag");
    }
  } catch (error) {
    console.error("Migration failed:", error);
    Deno.exit(1);
  }
}

async function getAllCustomers() {
  if (isDryRun && !apiKey) {
    // Return mock data for dry-run mode when no API key is provided
    console.log("🔍 DRY RUN: Using mock customer data");
    return [{ external_id: "customer-1" }, { external_id: "customer-2" }, { external_id: "customer-3" }];
  }

  const customers = [];
  let page = 1;
  let hasMore = true;

  while (hasMore) {
    const response = await lago.customers.findAllCustomers({ page, per_page: 100 });

    // Handle different response structures
    const customerData = response.customers || response.data?.customers || response;
    if (Array.isArray(customerData)) {
      customers.push(...customerData);
      hasMore = customerData.length === 100;
    } else if (customerData && Array.isArray(customerData)) {
      customers.push(...customerData);
      hasMore = customerData.length === 100;
    } else {
      console.log("Unexpected response structure, stopping pagination");
      hasMore = false;
    }
    page++;
  }

  return customers;
}

async function findActiveSubscription(customerId: string, subscriptionId: string) {
  if (isDryRun && !apiKey) {
    // Return mock subscription for demo purposes (only for customer-1)
    if (customerId === "customer-1") {
      return {
        external_id: subscriptionId,
        customer: { external_id: customerId },
        status: "active",
        ending_at: "2025-12-31",
      };
    }
    return null;
  }

  try {
    const subscription = await lago.subscriptions.findSubscription(subscriptionId);
    // Check if subscription belongs to this customer and is active
    if (
      subscription.data?.subscription?.external_customer_id === customerId &&
      subscription.data?.subscription?.status === "active"
    ) {
      return subscription;
    }

    return null;
  } catch (error: any) {
    // Subscription not found or invalid - both 404 and 422 should be treated as "no subscription"
    if (error.status === 404 || error.status === 422) {
      console.log(`    ℹ️  Subscription ${subscriptionId} not found or invalid (${error.status})`);
      return null;
    }
    throw error;
  }
}

async function checkForPendingLaunchTrial(customerId: string) {
  if (isDryRun && !apiKey) {
    // Mock: customer-2 has a pending launch_trial subscription
    if (customerId === "customer-2") {
      return {
        external_id: `${customerId}_launch_trial`,
        status: "pending",
        plan_code: "launch_trial",
      };
    }
    return null;
  }

  try {
    const launchTrialSubId = `${customerId}_launch_trial`;
    const subscription = await lago.subscriptions.findSubscription(launchTrialSubId, { status: "pending" });

    // Check if it's a pending or active launch_trial subscription
    const subData = subscription.data?.subscription;
    if (
      subData?.external_customer_id === customerId &&
      subData?.plan_code === "launch_trial" &&
      (subData?.status === "pending" || subData?.status === "active")
    ) {
      return subData;
    }

    return null;
  } catch (error: any) {
    // Subscription not found or invalid - both 404 and 422 should be treated as "no subscription"
    if (error.status === 404 || error.status === 422) {
      return null;
    }
    throw error;
  }
}

async function updateSubscriptionEndDate(subscriptionId: string, endDate: string) {
  return await lago.subscriptions.updateSubscription(subscriptionId, {
    subscription: {
      ending_at: endDate,
    },
  });
}

async function createLaunchTrialSubscription(customerId: string) {
  const startDate = new Date("2025-08-27");
  startDate.setUTCHours(0, 0, 0, 0);
  const formattedStartDate = `${startDate.toISOString().split(".")[0]}Z`;

  const endDate = new Date("2025-09-26"); // 30 days after start date
  endDate.setUTCHours(0, 0, 0, 0);
  const formattedEndDate = `${endDate.toISOString().split(".")[0]}Z`;

  const external_id = `${customerId}_launch_trial`;
  return await lago.subscriptions.createSubscription({
    subscription: {
      external_id,
      external_customer_id: customerId,
      plan_code: "launch_trial",
      subscription_at: formattedStartDate,
      ending_at: formattedEndDate,
    },
  });
}

// Run the migration
if (import.meta.main) {
  await migrateSubscriptions();
}
