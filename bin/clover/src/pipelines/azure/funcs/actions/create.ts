async function main(component: Input): Promise<Output> {
  const tenantId = requestStorage.getEnv("AZURE_TENANT_ID");
  const clientId = requestStorage.getEnv("AZURE_CLIENT_ID");
  const clientSecret = requestStorage.getEnv("AZURE_CLIENT_SECRET");

  if (!tenantId || !clientId || !clientSecret) {
    throw new Error("Azure credentials not found");
  }

  if (component.properties.resource?.payload) {
    return {
      status: "error",
      message: "Resource already exists",
      payload: component.properties.resource.payload,
    };
  }

  const domain = component.properties.domain;
  const apiVersion = _.get(domain, ["extra", "apiVersion"], "2023-01-01");

  const resourceId = createResourceId(domain);
  const token = await getAzureToken(tenantId, clientId, clientSecret);
  const url =
    `https://management.azure.com${resourceId}?api-version=${apiVersion}`;

  const createPayload = cleanPayload(domain);

  console.log(`[CREATE] Starting create operation for resource type: ${domain?.extra?.AzureResourceType}, resourceId: ${resourceId}`);
  console.log(`[CREATE] PUT ${url}`);

  const response = await fetch(url, {
    method: "PUT",
    headers: {
      "Authorization": `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify(createPayload),
  });

  console.log(`[CREATE] Response status: ${response.status}`);

  if (!response.ok) {
    const errorText = await response.text();
    console.error(`[CREATE] Failed with status ${response.status}:`, errorText);
    throw new Error(
      `Azure API Error: ${response.status} ${response.statusText} - ${errorText}`,
    );
  }

  // Handle Azure Long-Running Operations (LRO)
  // 202 is always async, 201 is async only if polling headers are present
  if (response.status === 201 || response.status === 202) {
    // Get the polling URL - prefer Azure-AsyncOperation over Location
    const asyncOpUrl = response.headers.get("Azure-AsyncOperation");
    const locationUrl = response.headers.get("Location");

    // If no polling headers, treat as synchronous completion (especially for 201)
    if (!asyncOpUrl && !locationUrl) {
      if (response.status === 201) {
        console.log(`[CREATE] 201 response with no polling headers, treating as synchronous completion`);
        const result = await parseJsonResponse(response, "CREATE");
        console.log(`[CREATE] Synchronous create successful, resourceId: ${result.id}`);
        return {
          payload: result,
          resourceId: result.id,
          status: "ok",
        };
      } else {
        // 202 without polling headers is an error
        throw new Error("LRO response missing polling URL headers");
      }
    }

    console.log(`[CREATE] LRO detected (${response.status}), polling for completion...`);

    // Azure-AsyncOperation returns status in body, Location returns 200 when done
    const isAsyncOpPattern = !!asyncOpUrl;
    let pollingUrl = asyncOpUrl || locationUrl;

    // Location header can be relative (starting with /) or absolute
    // Convert relative URLs to absolute
    if (pollingUrl && pollingUrl.startsWith('/')) {
      pollingUrl = `https://management.azure.com${pollingUrl}`;
      console.log(`[CREATE] Converted relative Location URL to absolute: ${pollingUrl}`);
    }

    console.log(`[CREATE] Using ${isAsyncOpPattern ? 'Azure-AsyncOperation' : 'Location'} polling pattern`);

    // Poll until the operation completes
    const finalResource = await pollLRO(pollingUrl, token, resourceId, apiVersion, isAsyncOpPattern);

    console.log(`[CREATE] Returning success with resourceId: ${finalResource.id}`);
    return {
      payload: finalResource,
      resourceId: finalResource.id,
      status: "ok",
    };
  }

  // Handle synchronous 200 response
  const result = await parseJsonResponse(response, "CREATE");

  console.log(`[CREATE] Synchronous create successful, resourceId: ${result.id}`);
  return {
    payload: result,
    resourceId: result.id,
    status: "ok",
  };
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

  // Remove metadata fields that are used for URL construction only
  if (domain.extra?.resourceId) {
    for (const key of domain.extra.resourceId.matchAll(/{([^}]+)}/g)) {
      const paramName = key[1];
      delete payload[paramName];
    }
  }
  delete payload.extra;

  // Merge discriminator subtypes into parent
  // propUsageMap.discriminators maps discriminator property -> subtype name -> enum value
  // e.g., { "kind": { "AzurePowerShellScript": "AzurePowerShell", "AzureCliScript": "AzureCLI" } }
  for (
    const [discriminatorProp, subtypeMap] of Object.entries(
      propUsageMap.discriminators || {},
    )
  ) {
    const discriminatorObject = payload[discriminatorProp];

    if (!discriminatorObject || typeof discriminatorObject !== "object") {
      continue;
    }

    // Find which subtype is filled in
    const filledSubtypes = Object.keys(subtypeMap).filter((subtype) =>
      discriminatorObject[subtype]
    );

    if (filledSubtypes.length > 1) {
      throw new Error(
        `Multiple discriminator subtypes filled for "${discriminatorProp}": ${filledSubtypes.join(", ")
        }. Only one should be filled.`,
      );
    }

    if (filledSubtypes.length === 0) {
      // No subtype filled in, remove the discriminator property
      delete payload[discriminatorProp];
      continue;
    }

    const subtypeName = filledSubtypes[0];
    const subtypeValue = discriminatorObject[subtypeName];

    if (
      subtypeValue && typeof subtypeValue === "object" &&
      !Array.isArray(subtypeValue)
    ) {
      // Merge subtype properties into parent (subtype values take precedence)
      Object.assign(payload, subtypeValue);

      // Set the discriminator property to the enum value from the map
      payload[discriminatorProp] = subtypeMap[subtypeName];
    } else {
      // If no subtype data, remove the discriminator property
      delete payload[discriminatorProp];
    }
  }

  // Only check top-level properties - once a property is included, keep all its descendants
  const propsToVisit = _.keys(payload).map((k: string) => [k]);

  while (propsToVisit.length > 0) {
    const key = propsToVisit.pop();

    let parent = payload;
    let keyOnParent = key[0];
    for (let i = 1; i < key.length; i++) {
      parent = parent[key[i - 1]];
      keyOnParent = key[i];
    }

    // Only check against propUsageMap for top-level domain properties
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

// Pulls resource ID from the domain
function createResourceId(domain: Input["properties"]["domain"]): string {
  const path = domain.extra?.resourceId;
  if (!path) {
    throw new Error("domain.extra.resourcePath is empty");
  }
  // Replace each instance of {paramName} with the corresponding domain property
  return path.replace(/{([^}]+)}/g, (_, paramName) => {
    const value = domain[paramName];
    if (!value) {
      throw new Error(`Domain property "${paramName}" is required`);
    }
    delete domain[paramName];
    return value;
  });
}

// Helper to safely parse JSON responses
async function parseJsonResponse(response: Response, context: string): Promise<any> {
  const responseText = await response.text();

  if (!responseText || responseText.trim() === "") {
    console.log(`[${context}] Empty response body, returning empty object`);
    return {};
  }

  try {
    return JSON.parse(responseText);
  } catch (parseError) {
    console.error(`[${context}] Failed to parse JSON response`);
    console.error(`[${context}] Response text: ${responseText}`);
    throw new Error(`Failed to parse ${context} response: ${parseError.message}`);
  }
}

async function pollLRO(
  pollingUrl: string,
  token: string,
  resourceId: string,
  apiVersion: string,
  isAsyncOpPattern: boolean,
): Promise<any> {
  const delay = (time: number) => {
    return new Promise((res) => {
      setTimeout(res, time);
    });
  };

  let finished = false;
  let success = false;
  let attempt = 0;
  const baseDelay = 1000;
  const maxDelay = 90000;
  let message = "";
  let finalResource = null;

  console.log(`[LRO] Starting status polling for operation: ${pollingUrl}`);

  while (!finished) {
    console.log(`[LRO] Status poll attempt ${attempt + 1}`);

    const statusResponse = await fetch(pollingUrl, {
      method: "GET",
      headers: {
        "Authorization": `Bearer ${token}`,
      },
    });

    // Location pattern: 202 means still running, 200 means done
    if (!isAsyncOpPattern) {
      if (statusResponse.status === 200) {
        console.log(`[LRO] Location polling complete with 200 OK`);
        finished = true;
        success = true;

        // Response body contains the final resource
        finalResource = await parseJsonResponse(statusResponse, "LRO-Location");
        console.log(`[LRO] Got final resource from Location polling with ID: ${finalResource.id}`);
      } else if (statusResponse.status === 202) {
        console.log(`[LRO] Location polling: operation still in progress (202)`);
        // Continue polling
      } else {
        console.error(`[LRO] Location poll failed: ${statusResponse.status} ${statusResponse.statusText}`);
        throw new Error(
          `LRO Location polling failed: ${statusResponse.status} ${statusResponse.statusText}`,
        );
      }
    } else {
      // Azure-AsyncOperation pattern: check status field in body
      if (!statusResponse.ok) {
        console.error(`[LRO] Status poll ${attempt + 1} failed: ${statusResponse.status} ${statusResponse.statusText}`);
        throw new Error(
          `LRO polling failed: ${statusResponse.status} ${statusResponse.statusText}`,
        );
      }

      const statusResult = await parseJsonResponse(statusResponse, "LRO-AsyncOp");
      const status = statusResult.status?.toLowerCase();

      console.log(`[LRO] Status poll ${attempt + 1} response: ${status}`);

      if (status === "succeeded") {
        console.log(`[LRO] Operation SUCCEEDED! Fetching final resource...`);
        finished = true;
        success = true;

        // Fetch the final resource to get the complete payload
        const resourceUrl = `https://management.azure.com${resourceId}?api-version=${apiVersion}`;

        const resourceResponse = await fetch(resourceUrl, {
          method: "GET",
          headers: {
            "Authorization": `Bearer ${token}`,
          },
        });

        if (!resourceResponse.ok) {
          throw new Error(
            `Failed to fetch created resource: ${resourceResponse.status} ${resourceResponse.statusText}`,
          );
        }

        finalResource = await parseJsonResponse(resourceResponse, "LRO-FinalResource");
        console.log(`[LRO] Successfully fetched final resource with ID: ${finalResource.id}`);
      } else if (status === "failed") {
        console.log(`[LRO] Operation FAILED: ${JSON.stringify(statusResult.error || statusResult)}`);
        finished = true;
        success = false;
        message = JSON.stringify(statusResult.error || statusResult);
      } else if (status === "canceled") {
        console.log(`[LRO] Operation CANCELLED`);
        finished = true;
        success = false;
        message = "Operation cancelled by Azure";
      }
    }

    if (!finished) {
      attempt++;
      const exponentialDelay = Math.min(baseDelay * Math.pow(2, attempt - 1), maxDelay);
      const jitter = Math.random() * 0.3 * exponentialDelay;
      const finalDelay = exponentialDelay + jitter;

      console.log(`[LRO] Waiting ${Math.round(finalDelay)}ms before status poll attempt ${attempt + 1}`);
      await delay(finalDelay);
    }
  }

  console.log(`[LRO] Final result: success=${success}`);

  if (success) {
    return finalResource;
  } else {
    throw new Error(`LRO ${message}`);
  }
}

async function getAzureToken(
  tenantId: string,
  clientId: string,
  clientSecret: string,
): Promise<string> {
  const tokenUrl =
    `https://login.microsoftonline.com/${tenantId}/oauth2/v2.0/token`;
  const body = new URLSearchParams({
    client_id: clientId,
    client_secret: clientSecret,
    scope: "https://management.azure.com/.default",
    grant_type: "client_credentials",
  });

  const response = await fetch(tokenUrl, {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: body.toString(),
  });

  if (!response.ok) {
    throw new Error(
      `Failed to get Azure token: ${response.status} ${response.statusText}`,
    );
  }

  const data = await response.json();
  return data.access_token;
}
