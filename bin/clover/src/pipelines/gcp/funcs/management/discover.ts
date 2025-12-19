async function main({ thisComponent }: Input): Promise<Output> {
  const component = thisComponent;

  // Get API path metadata from domain.extra
  const listApiPathJson = _.get(
    component.properties,
    ["domain", "extra", "listApiPath"],
    "",
  );

  if (!listApiPathJson) {
    return {
      status: "error",
      message: "No list API path metadata found - this resource may not support discovery",
    };
  }

  const listApiPath = JSON.parse(listApiPathJson);
  const getApiPathJson = _.get(
    component.properties,
    ["domain", "extra", "getApiPath"],
    "",
  );

  if (!getApiPathJson) {
    return {
      status: "error",
      message: "No get API path metadata found - this resource may not support discovery",
    };
  }

  const getApiPath = JSON.parse(getApiPathJson);
  const baseUrl = _.get(component.properties, ["domain", "extra", "baseUrl"], "");
  const gcpResourceType = _.get(
    component.properties,
    ["domain", "extra", "GcpResourceType"],
    "",
  );

  // Get authentication token
  const serviceAccountJson = requestStorage.getEnv("GOOGLE_APPLICATION_CREDENTIALS_JSON");
  if (!serviceAccountJson) {
    throw new Error("Google Cloud Credential not found. Please ensure a Google Cloud Credential is attached to this component.");
  }

  const { token, projectId } = await getAccessToken(serviceAccountJson);

  console.log(`Discovering ${gcpResourceType} resources...`);

  // Build refinement filter from domain properties
  const refinement = _.cloneDeep(thisComponent.properties.domain);
  delete refinement["extra"];
  // Remove any empty values, as they are never refinements
  for (const [key, value] of Object.entries(refinement)) {
    if (_.isEmpty(value)) {
      delete refinement[key];
    } else if (_.isPlainObject(value)) {
      refinement[key] = _.pickBy(
        value,
        (v) => !_.isEmpty(v) || _.isNumber(v) || _.isBoolean(v),
      );
      if (_.isEmpty(refinement[key])) {
        delete refinement[key];
      }
    }
  }

  // Build list URL by replacing path parameters
  let listUrl = `${baseUrl}${listApiPath.path}`;
  if (listApiPath.parameterOrder) {
    for (const paramName of listApiPath.parameterOrder) {
      let paramValue;

      if (paramName === "project") {
        paramValue = projectId;
      } else {
        paramValue = _.get(component.properties, ["domain", paramName]);
      }

      if (paramValue) {
        listUrl = listUrl.replace(`{${paramName}}`, encodeURIComponent(paramValue));
      }
    }
  }

  // Handle pagination with pageToken
  let resources: any[] = [];
  let nextPageToken: string | null = null;

  do {
    let currentUrl = listUrl;
    if (nextPageToken) {
      const separator = listUrl.includes("?") ? "&" : "?";
      currentUrl = `${listUrl}${separator}pageToken=${encodeURIComponent(nextPageToken)}`;
    }

    const listResponse = await siExec.withRetry(async () => {
      const resp = await fetch(currentUrl, {
        method: "GET",
        headers: {
          "Authorization": `Bearer ${token}`,
        },
      });

      if (!resp.ok) {
        const errorText = await resp.text();
        const error = new Error(`Google Cloud API Error: ${resp.status} ${resp.statusText} - ${errorText}`) as any;
        error.status = resp.status;
        error.body = errorText;
        throw error;
      }

      return resp;
    }, {
      isRateLimitedFn: (error) => error.status === 429
    }).then((r) => r.result);

    const listData = await listResponse.json();

    // GCP list responses typically have an "items" array
    const items = listData.items || [];
    resources = resources.concat(items);
    nextPageToken = listData.nextPageToken || null;

    if (nextPageToken) {
      console.log(`Fetching next page...`);
    }
  } while (nextPageToken);

  console.log(`Found ${resources.length} resources`);

  const create: Output["ops"]["create"] = {};
  const actions = {};
  let importCount = 0;

  for (const resource of resources) {
    // The resource ID is typically in the "name" or "id" field
    const resourceId = resource.name || resource.id || resource.selfLink;

    if (!resourceId) {
      console.log(`Skipping resource without ID`);
      continue;
    }

    console.log(`Importing ${resourceId}`);

    // Build the get URL to fetch full resource details
    let getUrl = `${baseUrl}${getApiPath.path}`;

    if (getApiPath.parameterOrder) {
      for (const paramName of getApiPath.parameterOrder) {
        let paramValue;

        // For the resource identifier, use resourceId
        if (paramName === getApiPath.parameterOrder[getApiPath.parameterOrder.length - 1]) {
          paramValue = resourceId;
        } else if (paramName === "project") {
          paramValue = projectId;
        } else {
          paramValue = _.get(resource, [paramName]) ||
                       _.get(component.properties, ["domain", paramName]);
        }

        if (paramValue) {
          getUrl = getUrl.replace(`{${paramName}}`, encodeURIComponent(paramValue));
        }
      }
    }

    // Fetch the full resource details with retry
    let resourceResponse;
    try {
      resourceResponse = await siExec.withRetry(async () => {
        const resp = await fetch(getUrl, {
          method: "GET",
          headers: {
            "Authorization": `Bearer ${token}`,
          },
        });

        if (!resp.ok) {
          const error = new Error(`Failed to fetch ${resourceId}`) as any;
          error.status = resp.status;
          throw error;
        }

        return resp;
      }, {
        isRateLimitedFn: (error) => error.status === 429
      }).then((r) => r.result);
    } catch (error) {
      console.log(`Failed to fetch ${resourceId} after retries, skipping`);
      continue;
    }

    const fullResource = await resourceResponse.json();

    const properties = {
      si: {
        resourceId,
      },
      domain: {
        ...component.properties?.domain || {},
        ...fullResource,
      },
      resource: fullResource,
    };

    // Apply refinement filter
    if (_.isEmpty(refinement) || _.isMatch(properties.domain, refinement)) {
      const newAttributes: Output["ops"]["create"][string]["attributes"] = {};
      for (const [skey, svalue] of Object.entries(component.sources || {})) {
        newAttributes[skey] = {
          $source: svalue,
        };
      }

      create[resourceId] = {
        kind: gcpResourceType || component.properties?.domain?.extra?.GcpResourceType,
        properties,
        attributes: newAttributes,
      };
      actions[resourceId] = {
        remove: ["create"],
      };
      importCount++;
    } else {
      console.log(
        `Skipping import of ${resourceId}; it did not match refinements`,
      );
    }
  }

  return {
    status: "ok",
    message: `Discovered ${importCount} ${gcpResourceType} resources`,
    ops: {
      create,
      actions,
    },
  };
}

async function getAccessToken(serviceAccountJson: string): Promise<{ token: string; projectId: string | undefined }> {
  // Parse service account JSON to extract project_id (optional)
  let projectId: string | undefined;
  try {
    const serviceAccount = JSON.parse(serviceAccountJson);
    projectId = serviceAccount.project_id;
  } catch {
    // If parsing fails or project_id is missing, continue without it
    projectId = undefined;
  }

  const activateResult = await siExec.waitUntilEnd("gcloud", [
    "auth",
    "activate-service-account",
    "--key-file=-",
    "--quiet"
  ], {
    input: serviceAccountJson
  });

  if (activateResult.exitCode !== 0) {
    throw new Error(`Failed to activate service account: ${activateResult.stderr}`);
  }

  const tokenResult = await siExec.waitUntilEnd("gcloud", [
    "auth",
    "print-access-token"
  ]);

  if (tokenResult.exitCode !== 0) {
    throw new Error(`Failed to get access token: ${tokenResult.stderr}`);
  }

  return {
    token: tokenResult.stdout.trim(),
    projectId,
  };
}
