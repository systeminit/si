async function main(_component: Input): Promise < Output > {
  const token = requestStorage.getEnv("HETZNER_API_TOKEN");
  if (!token) {
      return {
          result: "failure",
          message: 'Credentials are empty'
      };
  }
}