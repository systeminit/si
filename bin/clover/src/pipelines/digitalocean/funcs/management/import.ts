async function main({
  thisComponent,
}: Input): Promise<Output> {
  const component = thisComponent.properties;
  const token = requestStorage.getEnv("DO_API_TOKEN");
  if (!token) {
    throw new Error(
      "DO_API_TOKEN not found (hint: you may need a secret)",
    );
  }

  const endpoint = _.get(
    component,
    ["domain", "extra", "endpoint"],
    "",
  );
  const resourceId = _.get(component, ["si", "resourceId"], "");

  const scalarPropertyMapJson = _.get(component, [
    "domain",
    "extra",
    "ScalarPropertyMap",
  ], "[]");
  const propUsageMapJson = _.get(component, [
    "domain",
    "extra",
    "PropUsageMap",
  ], "{}");

  if (!endpoint) {
    throw new Error("Endpoint not found in extra properties");
  }

  if (!resourceId) {
    throw new Error("Resource ID not found in si properties");
  }

  // Parse ScalarPropertyMap to know which properties should be normalized to scalars
  let scalarProperties: Set<string>;
  try {
    const scalarPropsArray = JSON.parse(scalarPropertyMapJson);
    scalarProperties = new Set(scalarPropsArray);
  } catch (e) {
    console.warn("Failed to parse ScalarPropertyMap, using empty set:", e);
    scalarProperties = new Set();
  }

  // Parse PropUsageMap to know which properties are writable
  let writableProperties: Set<string>;
  try {
    const propUsageMap = JSON.parse(propUsageMapJson);
    writableProperties = new Set([
      ...(propUsageMap.createOnly || []),
      ...(propUsageMap.updatable || []),
    ]);
  } catch (e) {
    console.warn("Failed to parse PropUsageMap, using empty set:", e);
    writableProperties = new Set();
  }

  const url = `https://api.digitalocean.com${endpoint}/${resourceId}`;

  console.log(`Running request to  "${url}"`)

  const response = await fetch(
    url, {
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json",
      },
    },
  );

  if (!response.ok) {
    throw new Error(`API Error: ${response.status} ${response.statusText}`);
  }

  const data = await response.json();

  if (typeof data !== "object") {
    throw new Error("Unexpected response from API: not an object");
  }

  const resource = Object.values(data)[0];

  if (!resource) {
    console.log(JSON.stringify(data, null, 2));
    throw new Error("Unexpected response from API: no resource in object");
  }

  // Filter domain properties to only include writable properties (createOnly + updatable)
  const normalizedResource = normalizeForSchema(resource, scalarProperties);
  const domainProperties: Record<string, any> = {};

  for (const [key, value] of Object.entries(normalizedResource)) {
    if (writableProperties.has(key) && value != null) {
      domainProperties[key] = value;
    }
  }

  const properties = {
    ...component,
    domain: {
      ...component.domain,
      ...domainProperties,
    },
    resource: normalizedResource
  };


  return {
    status: "ok",
    message: `Imported ${endpoint} ${resourceId}`,
    ops: {
      update: {
        self: {
          properties,
        },
      },
      actions: {
        self: {
          // Remove any create actions, since this function sets the resource data directly
          remove: ["create"],
          add: [],
        },
      },
    },
  };
}

// Normalize API response to match schema expectations
// Extracts IDs from nested objects that should be scalars
// Recursively processes nested objects and arrays
function normalizeForSchema(
  obj: any,
  scalarProperties: Set<string>,
  isRootLevel = true,
): any {
  if (!obj || typeof obj !== "object") {
    return obj;
  }

  if (Array.isArray(obj)) {
    return obj.map((item) => normalizeForSchema(item, scalarProperties, false));
  }

  const normalized: any = {};
  for (const [key, value] of Object.entries(obj)) {
    if (value && typeof value === "object" && !Array.isArray(value)) {
      const nestedObj = value as any;

      // Root-level properties: extract slug/name/id if in ScalarPropertyMap
      if (isRootLevel && scalarProperties.has(key)) {
        if (nestedObj.slug !== undefined) {
          normalized[key] = nestedObj.slug;
        } else if (nestedObj.name !== undefined) {
          normalized[key] = nestedObj.name;
        } else if (nestedObj.id !== undefined) {
          normalized[key] = nestedObj.id;
        } else {
          normalized[key] = normalizeForSchema(value, scalarProperties, false);
        }
      } else {
        // Nested properties: recursively normalize but extract IDs from objects
        // that have an 'id' field (common pattern for DigitalOcean API)
        normalized[key] = normalizeNestedObject(value);
      }
    } else if (Array.isArray(value)) {
      normalized[key] = normalizeForSchema(value, scalarProperties, false);
    } else {
      normalized[key] = value;
    }
  }
  return normalized;
}

// Recursively normalize nested objects, extracting IDs where appropriate
function normalizeNestedObject(obj: any): any {
  if (!obj || typeof obj !== "object") {
    return obj;
  }

  if (Array.isArray(obj)) {
    return obj.map((item) => normalizeNestedObject(item));
  }

  // If object has only 'id' or 'slug', extract just that value
  // This handles cases like region: {slug: "nyc1", ...} -> "nyc1"
  if (obj.slug !== undefined && typeof obj.slug === "string") {
    return obj.slug;
  }
  if (obj.id !== undefined && typeof obj.id === "number") {
    return obj.id;
  }

  // Otherwise, recursively normalize the object's properties
  const normalized: any = {};
  for (const [key, value] of Object.entries(obj)) {
    if (value && typeof value === "object") {
      normalized[key] = normalizeNestedObject(value);
    } else {
      normalized[key] = value;
    }
  }
  return normalized;
}
