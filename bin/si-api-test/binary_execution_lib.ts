import { parse } from "https://deno.land/std@0.201.0/flags/mod.ts";
import assert from "node:assert";

export interface TestExecutionProfile {
  maxDuration: number;
  rate: number; // Time in between test runs
  useJitter: boolean; // If true, add a random amount of time to the rate, to avoid thundering herds
}

export function parseArgs(args: string[]) {
  // Parse arguments using std/flags
  const parsedArgs = parse(args, {
    string: [
      "workspaceId",
      "userId",
      "password",
      "profile",
      "tests",
      "token",
      "reportFile",
    ],
    alias: {
      w: "workspaceId",
      u: "userId",
      p: "password",
      t: "tests",
      l: "profile",
      k: "token",
    },
    default: { profile: undefined, tests: "" },
    boolean: ["help"],
  });

  // Display help information if required arguments are missing or help flag is set
  if (
    parsedArgs.help || !parsedArgs.workspaceId ||
    (!parsedArgs.userId && !parsedArgs.token)
  ) {
    console.log(`
Usage: deno run main.ts [options]

Options:
  --workspaceId, -w   Workspace ID (required)
  --userId, -u        User ID (required if token not set)
  --password, -p      User password (optional)
  --tests, -t         Test names to run (comma-separated, optional)
  --profile, -l       Test profile in JSON format (optional)
  --token, -k         SDF Auth Token (optional)
  --reportFile        Address of the output file, if unset it will be logged
  --help              Show this help message
`);
    Deno.exit(0);
  }

  // Extract parsed arguments
  const workspaceId = parsedArgs.workspaceId;
  const userId = parsedArgs.userId || undefined;
  const password = parsedArgs.password || undefined;
  const token = parsedArgs.token || undefined;
  const reportFile = parsedArgs.reportFile;

  // Handle optional tests argument
  const testsToRun = parsedArgs.tests
    ? parsedArgs.tests.split(",").map((test) => test.trim()).filter((test) =>
      test
    )
    : [];

  // Parse profile JSON if provided, otherwise the profile is one shot [aka single execution]
  let testProfile;
  if (parsedArgs.profile) {
    try {
      testProfile = JSON.parse(parsedArgs.profile) as TestExecutionProfile;
      assert(testProfile.maxDuration, "maxDuration is required on profile");
      assert(testProfile.rate, "rate is required on profile");
    } catch (error) {
      throw new Error(`Failed to parse profile JSON: ${error.message}`);
    }
  }

  return {
    workspaceId,
    userId,
    password,
    testsToRun,
    testProfile,
    token,
    reportFile,
  };
}

export function checkEnvironmentVariables(
  env: Record<string, string | undefined>,
) {
  const requiredVars = ["SDF_API_URL", "AUTH_API_URL"];
  const missingVars = requiredVars.filter((varName) =>
    !env[varName] || env[varName]?.length === 0
  );

  if (missingVars.length > 0) {
    throw new Error(`Missing environment variables: ${missingVars.join(", ")}`);
  }
}
