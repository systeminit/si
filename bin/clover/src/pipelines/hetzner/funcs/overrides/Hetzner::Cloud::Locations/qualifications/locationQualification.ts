async function main({ domain: { name } }: Input): Promise<Output> {
  const token = requestStorage.getEnv("HETZNER_API_TOKEN");
  if (!token) {
    return {
      result: "failure",
      message: "Credentials are empty",
    };
  }

  const response = await fetch(
    `https://api.hetzner.cloud/v1/locations?${new URLSearchParams({ name })}`,
    {
      headers: {
        Authorization: `Bearer ${token}`,
      },
    },
  );

  if (!response.ok) {
    return {
      result: "failure",
      message: "Credentials are invalid!",
    };
  }
  const result = await response.json();
  if (result.locations.length !== 1) {
    return {
      result: "failure",
      message: `Location "${name}" invalid (${result.locations.length} results)!`,
    };
  }

  return {
    result: "success",
  };
}
