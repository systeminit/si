type Input = {
  AssumeRole?: string;
  AccessKeyId?: string;
  SecretAccessKey?: string;
  SessionToken?: string;
  Endpoint?: string;
  WorkspaceToken?: string;
};

type Output = void;

async function main(secret: Input): Promise<Output> {
  // assume role and set returned creds as env var
  if (secret.AssumeRole) {
    let child;

    // if they've set keys, use them, otherwise use the si-access-prod profile
    if (secret.AccessKeyId || secret.SecretAccessKey) {
      child = await siExec.waitUntilEnd("aws", [
        "configure",
        "set",
        "aws_access_key_id",
        secret.AccessKeyId as string,
      ]);

      child = await siExec.waitUntilEnd("aws", [
        "configure",
        "set",
        "aws_secret_access_key",
        secret.SecretAccessKey as string,
      ]);

      child = await siExec.waitUntilEnd("aws", [
        "sts",
        "assume-role",
        "--role-arn",
        secret.AssumeRole as string,
        "--role-session-name",
        `SI_AWS_ACCESS_${secret.WorkspaceToken}`,
        "--external-id",
        secret.WorkspaceToken as string,
      ]);
    } else {
      child = await siExec.waitUntilEnd("aws", [
        "sts",
        "assume-role",
        "--role-arn",
        secret.AssumeRole as string,
        "--role-session-name",
        `SI_AWS_ACCESS_${secret.WorkspaceToken}`,
        "--external-id",
        secret.WorkspaceToken as string,
        "--profile",
        "si-access-prod",
      ]);
    }

    if (child.exitCode !== 0) {
      console.error(child.stderr);
      return;
    }

    const creds = JSON.parse(child.stdout).Credentials;

    requestStorage.setEnv("AWS_ACCESS_KEY_ID", creds.AccessKeyId);
    requestStorage.setEnv("AWS_SECRET_ACCESS_KEY", creds.SecretAccessKey);
    requestStorage.setEnv("AWS_SESSION_TOKEN", creds.SessionToken);
  } else {
    requestStorage.setEnv("AWS_ACCESS_KEY_ID", secret.AccessKeyId);
    requestStorage.setEnv("AWS_SECRET_ACCESS_KEY", secret.SecretAccessKey);
    if (secret.SessionToken) {
      requestStorage.setEnv("AWS_SESSION_TOKEN", secret.SessionToken);
    }
  }

  if (secret.Endpoint) {
    requestStorage.setEnv("AWS_ENDPOINT_URL", secret.Endpoint);
  }
}

export default main;
