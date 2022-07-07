import { Command } from "commander";
import { Worker } from "@temporalio/worker";

import { createActivities } from "./activities";

async function main() {
  let bindUds = "";
  let taskQueue = "";

  const program = new Command();
  program
    .version("0.0.1")
    .option(
      "--bind-uds <sock>",
      "modat unix domain socket path",
      "/var/run/modat.sock",
    )
    .option(
      "--task-queue <queue>",
      "the task queue the worker will pull from",
      "unknown",
    )
    .action((options) => {
      if (options.bindUds) {
        bindUds = options.bindUds;
      } else {
        console.error(`Missing required option --bind-uds`);
        process.exit(1);
      }
      if (options.taskQueue) {
        taskQueue = options.taskQueue;
      } else {
        console.error(`Missing required option --task-queue`);
        process.exit(1);
      }
    })
    .parse(process.argv);

  await run(taskQueue, bindUds);
}

async function run(taskQueue: string, bindUds: string) {
  const ctx = { bindUds };

  // Step 1: Register Workflows and Activities with the Worker and connect to
  // the Temporal server.
  const worker = await Worker.create({
    activities: createActivities(ctx),
    sinks: {
      logger: {
        info: {
          fn(workflowInfo, message, data?) {
            console.log(
              "workflow:",
              workflowInfo.runId,
              "message:",
              message,
              ...(data ? [JSON.stringify(data)] : []),
            );
          },
        },
      },
    },
    taskQueue,
    workflowsPath: require.resolve("./workflows"),
  });
  // Worker connects to localhost by default and uses console.error for logging.
  // Customize the Worker by passing more options to create():
  // https://typescript.temporal.io/api/classes/worker.Worker
  // If you need to configure server connection parameters, see docs:
  // https://docs.temporal.io/typescript/security#encryption-in-transit-with-mtls

  // Step 2: Start accepting tasks on the `hello-world` queue
  await worker.run();
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
