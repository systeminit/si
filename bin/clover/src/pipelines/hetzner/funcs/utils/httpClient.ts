async function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export async function hetznerGet(endpoint: string): Promise<any> {
  const apiToken = requestStorage.getEnv("HETZNER_API_TOKEN");
  if (!apiToken) {
    throw new Error("HETZNER_API_TOKEN not found");
  }

  const url = `https://api.hetzner.cloud/v1${endpoint}`;

  for (let attempt = 0; attempt < 3; attempt++) {
    const response = await fetch(url, {
      headers: {
        "Authorization": `Bearer ${apiToken}`,
        "Content-Type": "application/json",
      },
    });

    if (response.status === 429) {
      const resetTime = response.headers.get("RateLimit-Reset");
      const waitMs = resetTime ? (parseInt(resetTime) * 1000 - Date.now()) : 1000;
      await sleep(Math.max(waitMs, 1000));
      continue;
    }

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    return await response.json();
  }

  throw new Error("Rate limit exceeded after retries");
}