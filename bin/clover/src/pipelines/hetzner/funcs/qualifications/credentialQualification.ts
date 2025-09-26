async function main(_component: Input): Promise<Output> {
  const token = requestStorage.getEnv("HETZNER_API_TOKEN");
  if (!token) {
    return {
      result: "failure",
      message: "Credentials are empty",
    };
  }

  const response = await fetch(
    "https://api.hetzner.cloud/v1/locations",
    {
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json",
      },
    },
  );

  if (!response.ok) {
    return {
      result: "failure",
      message: "Credentials are invalid!",
    };
  }

  return {
    result: "success",
  };
}

