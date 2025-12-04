async function main({ thisComponent }: Input): Promise<Output> {
  const token = requestStorage.getEnv("DO_API_TOKEN");
  if (!token) {
    throw new Error(
      "DO_API_TOKEN not found (hint: you may need a secret)",
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
    "DigitalOceanResourceType",
  ], "");
  const identifierField = _.get(thisComponent.properties, [
    "domain",
    "extra",
    "IdentifierField",
  ], "id");
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
    throw new Error("DigitalOceanResourceType not found in extra properties");
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

  // Construct URL - endpoint already includes /v2/
  const url = `https://api.digitalocean.com${endpoint}`;

  const response = await fetch(
    url,
    {
      method: "GET",
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json",
      },
    },
  );

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`API Error: ${response.status} ${response.statusText}: ${errorText}`);
  }

  const data = await response.json();

  // Extract the payload from the response - it's the key that's not "links" or "meta"
  const resourceKey = Object.keys(data).find(key => key !== "links" && key !== "meta");
  const resources = resourceKey ? data[resourceKey] : [];

  if (!Array.isArray(resources)) {
    throw new Error(`Expected array of resources but got: ${typeof resources}`);
  }

  if (resources.length === 0) {
    console.log("No resources found, quitting");
    return {
      status: "ok",
      message: "No resources found",
      ops: {
        create,
        actions,
      },
    };
  }

  let importCount = 0;

  for (const resource of resources) {
    const resourceId = resource[identifierField]?.toString() || resource.name;
    console.log(`Importing ${resourceType} ${resourceId}`);

    // Normalize and clean the resource
    const normalizedResource = normalizeForSchema(resource, scalarProperties);
    const cleanedResource = removeNullValues(normalizedResource);

    const domainProperties: Record<string, any> = {};

    for (const [key, value] of Object.entries(cleanedResource)) {
      if (writableProperties.has(key)) {
        domainProperties[key] = value;
      }
    }

    const properties = {
      si: {
        resourceId,
        type: resourceType,
      },
      domain: domainProperties,
      resource: cleanedResource,
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
      add: ["refresh"], // Run refresh since listing endpoints don't return all fields
    };
    importCount += 1;
  }

  return {
    status: "ok",
    message: `Discovered ${importCount} ${resourceType}`,
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
        } else if (nestedObj.slug !== undefined) {
          normalized[key] = nestedObj.slug;
        } else {
          normalized[key] = normalizeForSchema(value, scalarProperties, false);
        }
      } else {
        // Nested properties: recursively normalize but extract IDs from objects
        // that have an 'id' field (common pattern for Digital Ocean API)
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
  if (obj === null) {
    return undefined;
  }

  if (!obj || typeof obj !== "object") {
    return obj;
  }

  if (Array.isArray(obj)) {
    return obj.map((item) => normalizeNestedObject(item));
  }

  // If object has only 'id' or both 'id' and simple metadata, extract just the id
  // This handles cases like image: {id: 123, name: "ubuntu", ...} -> 123
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

// Recursively remove null values from objects and arrays
// This prevents schema validation errors when API returns null for array/object fields
function removeNullValues(obj: any): any {
  if (obj === null || obj === undefined) {
    return undefined;
  }

  if (Array.isArray(obj)) {
    const filtered = obj.map(item => removeNullValues(item)).filter(item => item !== undefined);
    return filtered.length > 0 ? filtered : undefined;
  }

  if (typeof obj === 'object') {
    const cleaned: any = {};
    for (const [key, value] of Object.entries(obj)) {
      const cleanedValue = removeNullValues(value);
      if (cleanedValue !== undefined) {
        cleaned[key] = cleanedValue;
      }
    }
    return Object.keys(cleaned).length > 0 ? cleaned : undefined;
  }

  return obj;
}
