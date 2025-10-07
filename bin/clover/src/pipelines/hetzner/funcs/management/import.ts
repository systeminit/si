async function main({
  thisComponent,
}: Input): Promise<Output> {
  const component = thisComponent;
  const token = requestStorage.getEnv("HETZNER_API_TOKEN");
  if (!token) {
    throw new Error(
      "HETZNER_API_TOKEN not found (hint: you may need a secret)",
    );
  }

  const endpoint = _.get(
    component.properties,
    ["domain", "extra", "endpoint"],
    "",
  );
  const resourceType = _.get(
    component.properties,
    ["domain", "extra", "HetznerResourceType"],
    "",
  );
  const resourceId = _.get(component.properties, ["si", "resourceId"], "");
  const scalarPropertyMapJson = _.get(component.properties, [
    "domain",
    "extra",
    "ScalarPropertyMap",
  ], "[]");

  if (!endpoint) {
    throw new Error("Endpoint not found in extra properties");
  }

  if (!resourceType) {
    throw new Error("HetznerResourceType not found in extra properties");
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

  const response = await fetch(
    `https://api.hetzner.cloud/v1/${endpoint}/${resourceId}`,
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
  const resource = data[endpoint.slice(0, -1)] || data;

  console.log(`Importing ${endpoint} ${resourceId}`);

  const create: Output["ops"]["create"] = {};
  const actions = {};

  const properties = {
    si: {
      resourceId,
      type: resourceType,
    },
    domain: normalizeForSchema(resource, scalarProperties),
    resource: resource,
  };

  const newAttributes: Output["ops"]["create"][string]["attributes"] = {};
  for (const [skey, svalue] of Object.entries(component.sources || {})) {
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

  return {
    status: "ok",
    message: `Imported ${endpoint} ${resourceId}`,
    ops: {
      create,
      actions,
    },
  };
}

// Normalize API response to match schema expectations
// When API returns objects but schema expects scalars, extract name or id
// Only applies to root-level properties that Hetzner returns as objects
// but the schema defines as strings/numbers (determined from ScalarPropertyMap)
// Recursively processes nested objects and arrays while preserving structure
function normalizeForSchema(
  obj: any,
  scalarProperties: Set<string>,
  isRootLevel = true,
): any {
  if (!obj || typeof obj !== "object") {
    return obj;
  }

  // Handle arrays by recursively normalizing each element
  if (Array.isArray(obj)) {
    return obj.map((item) => normalizeForSchema(item, scalarProperties, false));
  }

  const normalized: any = {};
  for (const [key, value] of Object.entries(obj)) {
    if (value && typeof value === "object" && !Array.isArray(value)) {
      const nestedObj = value as any;

      // Only extract name/id for root-level properties that should be scalars
      if (isRootLevel && scalarProperties.has(key)) {
        if (nestedObj.name !== undefined) {
          normalized[key] = nestedObj.name;
        } else if (nestedObj.id !== undefined) {
          normalized[key] = nestedObj.id;
        } else {
          // Fallback: preserve structure if no name/id
          normalized[key] = normalizeForSchema(value, scalarProperties, false);
        }
      } else {
        // Preserve nested structure for all other properties
        normalized[key] = normalizeForSchema(value, scalarProperties, false);
      }
    } else if (Array.isArray(value)) {
      // Recursively normalize arrays
      normalized[key] = normalizeForSchema(value, scalarProperties, false);
    } else {
      normalized[key] = value;
    }
  }
  return normalized;
}
