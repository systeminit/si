async function main({ thisComponent }: Input): Promise<Output> {
  const token = requestStorage.getEnv("HETZNER_API_TOKEN");
  if (!token) {
    throw new Error(
      "HETZNER_API_TOKEN not found (hint: you may need a secret)",
    );
  }

  const endpoint = _.get(thisComponent.properties, [
    "domain",
    "extra",
    "endpoint",
  ], "");
  const resourceType = _.get(thisComponent.properties, [
    "domain",
    "extra",
    "HetznerResourceType",
  ], "");
  const scalarPropertyMapJson = _.get(thisComponent.properties, [
    "domain",
    "extra",
    "ScalarPropertyMap",
  ], "[]");
  const propUsageMapJson = _.get(thisComponent.properties, [
    "domain",
    "extra",
    "PropUsageMap",
  ], "{}");

  if (!endpoint) {
    throw new Error("Endpoint not found in extra properties");
  }

  if (!resourceType) {
    throw new Error("HetznerResourceType not found in extra properties");
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

  const create: Output["ops"]["create"] = {};
  const actions = {};

  const response = await fetch(
    `https://api.hetzner.cloud/v1/${endpoint}`,
    {
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
  const resources = data[endpoint] || [];
  let importCount = 0;

  for (const resource of resources) {
    const resourceId = resource.id?.toString() || resource.name;
    console.log(`Importing ${endpoint} ${resourceId}`);

    // Filter domain properties to only include writable properties (createOnly + updatable)
    // Also skip null values since they may not match the schema's type expectations
    const normalizedResource = normalizeForSchema(resource, scalarProperties);
    const domainProperties: Record<string, any> = {};

    for (const [key, value] of Object.entries(normalizedResource)) {
      if (writableProperties.has(key) && value != null) {
        domainProperties[key] = value;
      }
    }

    const properties = {
      si: {
        resourceId,
        type: resourceType,
      },
      domain: domainProperties,
      resource: resource,
    };

    const newAttributes: Output["ops"]["create"][string]["attributes"] = {};
    for (const [skey, svalue] of Object.entries(thisComponent.sources)) {
      newAttributes[skey] = {
        $source: svalue,
      };
    }

    create[resourceId] = {
      kind: resourceType,
      properties,
      attributes: newAttributes,
    };
    actions[resourceId] = {
      remove: ["create"],
    };
    importCount += 1;
  }

  return {
    status: "ok",
    message: `Discovered ${importCount} ${endpoint}`,
    ops: {
      create,
      actions,
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

      // Root-level properties: extract name/id if in ScalarPropertyMap
      if (isRootLevel && scalarProperties.has(key)) {
        if (nestedObj.name !== undefined) {
          normalized[key] = nestedObj.name;
        } else if (nestedObj.id !== undefined) {
          normalized[key] = nestedObj.id;
        } else {
          normalized[key] = normalizeForSchema(value, scalarProperties, false);
        }
      } else {
        // Nested properties: recursively normalize but extract IDs from objects
        // that have an 'id' field (common pattern for Hetzner API)
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

  // If object has only 'id' or both 'id' and simple metadata, extract just the id
  // This handles cases like ipv4: {id: 123, ip: "1.2.3.4", ...} -> 123
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
