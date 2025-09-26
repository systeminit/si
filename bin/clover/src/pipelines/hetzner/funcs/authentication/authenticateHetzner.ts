async function main(secret: Input): Promise<Output> {
  requestStorage.setEnv("HETZNER_API_TOKEN", secret.HetznerApiToken);
}

