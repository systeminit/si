async function main(component: Input): Promise<Output> {
  const tenantId = requestStorage.getEnv("ENTRA_TENANT_ID") ||
    requestStorage.getEnv("AZURE_TENANT_ID");
  const clientId = requestStorage.getEnv("ENTRA_CLIENT_ID") ||
    requestStorage.getEnv("AZURE_CLIENT_ID");
  const clientSecret = requestStorage.getEnv("ENTRA_CLIENT_SECRET") ||
    requestStorage.getEnv("AZURE_CLIENT_SECRET");

  if (!tenantId || !clientId || !clientSecret) {
    throw new Error(
      "Microsoft credentials not found. Need ENTRA_TENANT_ID (or AZURE_TENANT_ID), ENTRA_CLIENT_ID (or AZURE_CLIENT_ID), and ENTRA_CLIENT_SECRET (or AZURE_CLIENT_SECRET)",
    );
  }

  if (component.properties.resource?.payload) {
    return {
      status: "error",
      message: "Resource already exists",
      payload: component.properties.resource.payload,
    };
  }

  const domain = component.properties.domain;
  const endpoint = _.get(domain, ["extra", "endpoint"], "");
  const apiVersion = _.get(domain, ["extra", "apiVersion"], "v1.0");

  if (!endpoint) {
    throw new Error("Missing endpoint in domain.extra.endpoint");
  }

  const token = await getGraphToken(tenantId, clientId, clientSecret);
  const url = `https://graph.microsoft.com/${apiVersion}/${endpoint}`;

  const createPayload = cleanPayload(domain);

  console.log(
    `[CREATE] Starting create operation for resource type: ${
      domain?.extra?.EntraResourceType
    }, endpoint: ${endpoint}`,
  );
  console.log(`[CREATE] POST ${url}`);

  const response = await fetch(url, {
    method: "POST",
    headers: {
      "Authorization": `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify(createPayload),
  });

  console.log(`[CREATE] Response status: ${response.status}`);

  if (!response.ok) {
    const errorText = await response.text();
    console.error(
      `[CREATE] Failed with status ${response.status}:`,
      errorText,
    );
    throw new Error(
      `Graph API Error: ${response.status} ${response.statusText} - ${errorText}`,
    );
  }

  // Handle Long-Running Operations (rare in Graph API but possible with status 202)
  if (response.status === 202) {
    console.log(`[CREATE] LRO detected (202), polling for completion...`);

    const locationUrl = response.headers.get("Location");
    if (!locationUrl) {
      throw new Error("LRO response missing Location header");
    }

    const finalResource = await pollLRO(locationUrl, token);

    console.log(
      `[CREATE] Returning success with resourceId: ${finalResource.id}`,
    );
    return {
      payload: finalResource,
      resourceId: finalResource.id,
      status: "ok",
    };
  }

  // Handle synchronous 200/201 response
  const result = await response.json();

  console.log(`[CREATE] Synchronous create successful, resourceId: ${result.id}`);
  return {
    payload: result,
    resourceId: result.id,
    status: "ok",
  };
}

async function getGraphToken(
  tenantId: string,
  clientId: string,
  clientSecret: string,
): Promise<string> {
  const tokenUrl =
    `https://login.microsoftonline.com/${tenantId}/oauth2/v2.0/token`;
  const body = new URLSearchParams({
    client_id: clientId,
    client_secret: clientSecret,
    scope: "https://graph.microsoft.com/.default",
    grant_type: "client_credentials",
  });

  const response = await fetch(tokenUrl, {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: body.toString(),
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(
      `Failed to get Graph API token: ${response.status} ${response.statusText} - ${errorText}`,
    );
  }

  const data = await response.json();
  return data.access_token;
}

function cleanPayload(domain) {
  const propUsageMap = JSON.parse(domain.extra.PropUsageMap);
  if (
    !Array.isArray(propUsageMap.createOnly) ||
    !Array.isArray(propUsageMap.updatable)
  ) {
    throw Error("malformed propUsageMap on resource");
  }

  const payload = _.cloneDeep(domain);

  // Remove extra metadata
  delete payload.extra;

  // Merge discriminator subtypes into parent
  for (
    const [discriminatorProp, subtypeMap] of Object.entries(
      propUsageMap.discriminators || {},
    )
  ) {
    const discriminatorObject = payload[discriminatorProp];

    if (!discriminatorObject || typeof discriminatorObject !== "object") {
      continue;
    }

    const filledSubtypes = Object.keys(subtypeMap).filter((subtype) =>
      discriminatorObject[subtype]
    );

    if (filledSubtypes.length > 1) {
      throw new Error(
        `Multiple discriminator subtypes filled for "${discriminatorProp}": ${
          filledSubtypes.join(", ")
        }. Only one should be filled.`,
      );
    }

    if (filledSubtypes.length === 0) {
      delete payload[discriminatorProp];
      continue;
    }

    const subtypeName = filledSubtypes[0];
    const subtypeValue = discriminatorObject[subtypeName];

    if (
      subtypeValue && typeof subtypeValue === "object" &&
      !Array.isArray(subtypeValue)
    ) {
      Object.assign(payload, subtypeValue);
      payload[discriminatorProp] = subtypeMap[subtypeName];
    } else {
      delete payload[discriminatorProp];
    }
  }

  // Only check top-level properties
  const propsToVisit = _.keys(payload).map((k: string) => [k]);

  while (propsToVisit.length > 0) {
    const key = propsToVisit.pop();

    let parent = payload;
    let keyOnParent = key[0];
    for (let i = 1; i < key.length; i++) {
      parent = parent[key[i - 1]];
      keyOnParent = key[i];
    }

    if (key.length === 1) {
      const propPath = `/domain/${keyOnParent}`;
      if (
        !propUsageMap.createOnly.includes(propPath) &&
        !propUsageMap.updatable.includes(propPath)
      ) {
        delete parent[keyOnParent];
        continue;
      }
    }

    const prop = parent[keyOnParent];

    if (typeof prop !== "object" || Array.isArray(prop)) {
      continue;
    }

    for (const childKey of _.keys(prop)) {
      propsToVisit.unshift([...key, childKey]);
    }
  }

  return extLib.removeEmpty(payload);
}

async function pollLRO(
  pollingUrl: string,
  token: string,
): Promise<any> {
  const delay = (time: number) => {
    return new Promise((res) => {
      setTimeout(res, time);
    });
  };

  let finished = false;
  let attempt = 0;
  const baseDelay = 1000;
  const maxDelay = 30000;

  console.log(`[LRO] Starting status polling for operation: ${pollingUrl}`);

  while (!finished) {
    console.log(`[LRO] Status poll attempt ${attempt + 1}`);

    const statusResponse = await fetch(pollingUrl, {
      method: "GET",
      headers: {
        "Authorization": `Bearer ${token}`,
      },
    });

    if (statusResponse.status === 200) {
      console.log(`[LRO] Operation complete with 200 OK`);
      const finalResource = await statusResponse.json();
      console.log(
        `[LRO] Got final resource with ID: ${finalResource.id || "unknown"}`,
      );
      return finalResource;
    } else if (statusResponse.status === 202) {
      console.log(`[LRO] Operation still in progress (202)`);
    } else {
      console.error(
        `[LRO] Poll failed: ${statusResponse.status} ${statusResponse.statusText}`,
      );
      const errorBody = await statusResponse.json();
      throw new Error(`LRO failed: ${JSON.stringify(errorBody)}`);
    }

    attempt++;
    const exponentialDelay = Math.min(
      baseDelay * Math.pow(2, attempt - 1),
      maxDelay,
    );
    const jitter = Math.random() * 0.3 * exponentialDelay;
    const finalDelay = exponentialDelay + jitter;

    console.log(
      `[LRO] Waiting ${Math.round(finalDelay)}ms before status poll attempt ${
        attempt + 1
      }`,
    );
    await delay(finalDelay);
  }
}
