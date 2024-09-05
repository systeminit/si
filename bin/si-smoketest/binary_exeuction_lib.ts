// This lib can be used for binary args management, i.e.
// parsing inputs
// setting default variables
// setting verbosity/logging levels
// interrogating the environment to figure out exactly how to execute the tests

export function parseArgs(args: string[]) {
    if (args.length < 2) {
      throw new Error("Expected at least 2 args: workspaceId, userEmail, and optionally userPassword and test names");
    }
  
    const workspaceId = args[0];
    const userId = args[1];
    const password = args[2] || undefined; // Password is optional
    const testsToRun = args.slice(3); // Skip the first three arguments
  
    return { workspaceId, userId, password, testsToRun };
}