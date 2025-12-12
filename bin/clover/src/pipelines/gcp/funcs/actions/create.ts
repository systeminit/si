async function main(component: Input): Promise<Output> {
  const serviceAccountJson = requestStorage.getEnv("GOOGLE_APPLICATION_CREDENTIALS_JSON");

  if (!serviceAccountJson) {
    throw new Error("Google Cloud Credential not found. Please ensure a Google Cloud Credential is attached to this component.");
  }

  // Activate service account using stdin (most secure - no file on disk)
  const activateResult = await siExec.waitUntilEnd("gcloud", [
    "auth",
    "activate-service-account",
    "--key-file=-",
    "--quiet"
  ], {
    input: serviceAccountJson
  });

  if (activateResult.exitCode !== 0) {
    throw new Error(`Failed to activate service account: ${activateResult.stderr}`);
  }

  // Get the access token
  const tokenResult = await siExec.waitUntilEnd("gcloud", [
    "auth",
    "print-access-token"
  ]);

  if (tokenResult.exitCode !== 0) {
    throw new Error(`Failed to get access token: ${tokenResult.stderr}`);
  }

  const token = tokenResult.stdout.trim();

  // TODO: Implement actual create logic using the token
  throw new Error("Not implemented");
}
