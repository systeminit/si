/**
 * Lago Subscription Migration Fixup Script
 *
 * This script adds a "pay_as_you_go" subscription to follow on from their `launch_trial` subscription
 *
 * What it does:
 * 1. Fetches all customers from Lago billing platform
 * 2. For each customer with an active "launch_trial" subscription:
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
  console.error(
    "Error: LAGO_API_KEY environment variable is required (unless running in dry-run mode)",
  );
  Deno.exit(1);
}

const lago = apiKey ? Client(apiKey, apiUrl) : null;

async function migrateSubscriptions() {
  try {
    console.log(
      `Starting subscription migration${isDryRun ? " (DRY RUN MODE)" : ""}...`,
    );
    if (isDryRun) {
      console.log(
        "🔍 DRY RUN: No actual changes will be made to subscriptions",
      );
    }

    if (batchSize !== Infinity) {
      console.log(
        `📦 BATCH MODE: Will process maximum ${batchSize} migrations (excluding skipped customers)`,
      );
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
        console.log(
          `🛑 Reached batch limit of ${batchSize} migrations. Stopping processing.`,
        );
        const remainingCustomers = customers.length - processedCount;
        if (remainingCustomers > 0) {
          console.log(
            `⏭️  ${remainingCustomers} remaining customers will be processed in next batch`,
          );
        }
        break;
      }
      processedCount++;
      console.log(
        `Processing customer ${processedCount}/${customers.length}: ${customer.external_id}`,
      );

      // Check for active launch_trial subscription
      const launchTrialSubId = `${customer.external_id}_launch_trial`;
      const activeSubscription = await findActiveSubscription(
        customer.external_id,
        launchTrialSubId,
      );

      if (activeSubscription) {
        const subData =
          activeSubscription.data?.subscription || activeSubscription;
        const subscriptionId = subData.external_id || subData.subscription_id;
        const currentEndDate =
          subData.ending_at || subData.terminated_at || "No end date";

        console.log(
          `  ✅ Found active pay_as_you_go subscription: ${subscriptionId}`,
        );

        // Check if customer already has a pending launch_pay_as_you_go subscription
        const existingPayAsYouGo = await checkForPendinPayAsYouGo(
          customer.external_id,
        );

        if (existingPayAsYouGo) {
          console.log(
            `  ⏩ SKIPPING: Customer already has ${existingPayAsYouGo.status} launch_pay_as_you_go subscription: ${existingPayAsYouGo.external_id}`,
          );
          skippedCount++;
          continue;
        }

        foundSubscriptions.push({
          customerId: customer.external_id,
          subscriptionId: subscriptionId,
          currentEndDate: currentEndDate,
          newEndDate: currentEndDate,
          newPayAsYouGoStart: "2025-09-26",
        });

        if (isDryRun) {
          console.log(
            `  🔍 DRY RUN: Would create new launch_pay_as_you_go subscription starting 2025-09-26`,
          );
        } else {
          // Step 3: Create new launch_pay_as_you_go subscription
          await creatPayAsYouGoSubscription(customer.external_id);
          console.log(`  ✅ Created new launch_pay_as_you_go subscription`);
        }

        migratedCount++;
      } else {
        console.log(`  ❌ No active launch_trial subscription found`);
      }
    }

    console.log(`\nMigration completed!`);
    console.log(`  Processed: ${processedCount} customers`);
    console.log(
      `  ${isDryRun ? "Found" : "Migrated"}: ${migratedCount} subscriptions`,
    );
    console.log(
      `  Skipped: ${skippedCount} customers (already have launch_pay_as_you_go)`,
    );

    // Print summary report for dry-run
    if (isDryRun && foundSubscriptions.length > 0) {
      console.log("\n📋 DRY RUN SUMMARY:");
      console.log("=".repeat(80));
      foundSubscriptions.forEach((sub, index) => {
        console.log(`${index + 1}. Customer: ${sub.customerId}`);
        console.log(`   Current subscription: ${sub.subscriptionId}`);
        console.log(`   Current end date: ${sub.currentEndDate}`);
        console.log(
          `   → Would create launch_pay_as_you_go: ${sub.newTrialStart}`,
        );
        console.log("");
      });
      console.log(
        `Total subscriptions to migrate: ${foundSubscriptions.length}`,
      );
      if (skippedCount > 0) {
        console.log(
          `Total customers skipped: ${skippedCount} (already have launch_pay_as_you_go subscriptions)`,
        );
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
    return [
      { external_id: "customer-1" },
      { external_id: "customer-2" },
      { external_id: "customer-3" },
    ];
  }

  const customers = [];
  let page = 1;
  let hasMore = true;

  while (hasMore) {
    const response = await lago.customers.findAllCustomers({
      page,
      per_page: 100,
    });

    // Handle different response structures
    const customerData =
      response.customers || response.data?.customers || response;
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

async function findActiveSubscription(
  customerId: string,
  subscriptionId: string,
) {
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
    const subscription =
      await lago.subscriptions.findSubscription(subscriptionId);
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
      console.log(
        `    ℹ️  Subscription ${subscriptionId} not found or invalid (${error.status})`,
      );
      return null;
    }
    throw error;
  }
}

async function checkForPendinPayAsYouGo(customerId: string) {
  if (isDryRun && !apiKey) {
    // Mock: customer-2 has a pending launch_trial subscription
    if (customerId === "customer-2") {
      return {
        external_id: `${customerId}_launch_pay_as_you_go`,
        status: "pending",
        plan_code: "launch_pay_as_you_go",
      };
    }
    return null;
  }

  try {
    const payAsYouGoSubId = `${customerId}_launch_pay_as_you_go`;
    const subscription = await lago.subscriptions.findSubscription(
      payAsYouGoSubId,
      { status: "pending" },
    );

    // Check if it's a pending or active launch_trial subscription
    const subData = subscription.data?.subscription;
    if (
      subData?.external_customer_id === customerId &&
      subData?.plan_code === "launch_pay_as_you_go" &&
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

async function creatPayAsYouGoSubscription(customerId: string) {
  const startDate = new Date("2025-09-26");
  startDate.setUTCHours(0, 0, 0, 0);
  const formattedStartDate = `${startDate.toISOString().split(".")[0]}Z`;

  const external_id = `${customerId}_launch_pay_as_you_go`;
  return await lago.subscriptions.createSubscription({
    subscription: {
      external_id,
      external_customer_id: customerId,
      plan_code: "launch_pay_as_you_go",
      subscription_at: formattedStartDate,
    },
  });
}

// Run the migration
if (import.meta.main) {
  await migrateSubscriptions();
}
