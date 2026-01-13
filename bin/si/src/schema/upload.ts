import { Context } from "../context.ts";
import { unknownValueToErrorMessage } from "../helpers.ts";
import { wrapInChangeSet } from "./change_set.ts";
import {
  ChangeSetsApi,
  type InstallFromFileV1Response,
} from "@systeminit/api-client";
import { getWorkspaceDetails } from "../cli/helpers.ts";

/**
 * Options for uploading a PkgSpec file
 */
export interface SchemaUploadOptions {
  /**
   * Path to the PkgSpec JSON file
   */
  filePath: string;

  /**
   * Change set ID or name (optional - will create a new one if not specified)
   */
  changeSet?: string;
}

/**
 * Uploads a PkgSpec JSON file to install a schema in a workspace.
 *
 * This command uses the luminork install_from_file endpoint which accepts
 * a multipart form with the PkgSpec JSON content.
 *
 * @param options - Options containing the file path and optional change set
 */
export async function callSchemaUpload(
  options: SchemaUploadOptions,
): Promise<void> {
  const ctx = Context.instance();
  const { filePath, changeSet } = options;

  ctx.logger.info(`Uploading PkgSpec from: ${filePath}`);
  ctx.logger.info("---");
  ctx.logger.info("");

  const apiConfig = Context.apiConfig();
  const workspaceId = Context.workspaceId();
  const changeSetsApi = new ChangeSetsApi(apiConfig);

  // Read the PkgSpec file
  let pkgSpecContent: string;
  try {
    pkgSpecContent = await Deno.readTextFile(filePath);
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) {
      ctx.logger.error(`File not found: ${filePath}`);
      throw new Error(`PkgSpec file not found: ${filePath}`);
    }
    throw error;
  }

  // Validate it's valid JSON
  try {
    JSON.parse(pkgSpecContent);
  } catch {
    ctx.logger.error(`Invalid JSON in file: ${filePath}`);
    throw new Error(`File is not valid JSON: ${filePath}`);
  }

  ctx.logger.info("PkgSpec file loaded successfully");

  // Determine change set
  let targetChangeSetId: string | undefined;

  if (changeSet) {
    // Try to find existing change set by name or use as ID
    const changeSetsResponse = await changeSetsApi.listChangeSets({
      workspaceId,
    });
    const changeSets = changeSetsResponse.data.changeSets as {
      id: string;
      name: string;
      isHead: boolean;
    }[];

    // Try to find by name first
    const foundCs = changeSets.find(
      (cs) =>
        cs.name.toLowerCase() === changeSet.toLowerCase() ||
        cs.id === changeSet,
    );

    if (foundCs) {
      if (foundCs.isHead) {
        ctx.logger.error(
          "Cannot upload spec to HEAD change set. Please specify or create a non-HEAD change set.",
        );
        throw new Error("Cannot upload spec to HEAD change set");
      }
      targetChangeSetId = foundCs.id;
      ctx.logger.info(`Using existing change set: ${foundCs.name}`);
    } else {
      ctx.logger.error(`Change set not found: ${changeSet}`);
      throw new Error(`Change set not found: ${changeSet}`);
    }
  }

  const workspaceDetails = await getWorkspaceDetails(workspaceId);

  if (targetChangeSetId) {
    // Use existing change set
    await uploadSpecToChangeSet(
      ctx,
      workspaceId,
      targetChangeSetId,
      pkgSpecContent,
      workspaceDetails.instanceUrl,
    );
  } else {
    // Create a new change set for the upload
    ctx.logger.info("Creating new change set for schema upload...");

    await wrapInChangeSet(
      changeSetsApi,
      ctx.logger,
      workspaceId,
      "SI Schema Upload",
      async (changeSetId) => {
        await uploadSpecToChangeSet(
          ctx,
          workspaceId,
          changeSetId,
          pkgSpecContent,
          workspaceDetails.instanceUrl,
        );
      },
    );
  }
}

/**
 * Uploads the PkgSpec content to a specific change set
 */
async function uploadSpecToChangeSet(
  ctx: Context,
  workspaceId: string,
  changeSetId: string,
  pkgSpecContent: string,
  instanceUrl: string,
): Promise<void> {
  const apiConfig = Context.apiConfig();

  // Build the multipart form data
  const formData = new FormData();
  const blob = new Blob([pkgSpecContent], { type: "application/json" });
  formData.append("pkg_spec", blob, "pkg_spec.json");

  // Build the URL for the luminork endpoint
  const baseUrl = apiConfig.basePath || instanceUrl;
  const url =
    `${baseUrl}/v1/w/${workspaceId}/change-sets/${changeSetId}/schemas/install_from_file`;

  ctx.logger.info(`Uploading to: ${url}`);

  try {
    // Make the request with the auth token
    const headers: HeadersInit = {};
    if (apiConfig.accessToken) {
      const token = typeof apiConfig.accessToken === "function"
        ? await apiConfig.accessToken()
        : apiConfig.accessToken;
      headers["Authorization"] = `Bearer ${token}`;
    }

    const response = await fetch(url, {
      method: "POST",
      headers,
      body: formData,
    });

    if (!response.ok) {
      const errorText = await response.text();
      let errorMessage: string;

      try {
        const errorJson = JSON.parse(errorText);
        errorMessage = errorJson.error?.message || errorJson.message ||
          errorText;
      } catch {
        errorMessage = errorText;
      }

      if (response.status === 400) {
        ctx.logger.error(`Bad request: ${errorMessage}`);
        throw new Error(`Upload failed: ${errorMessage}`);
      } else if (response.status === 401) {
        ctx.logger.error("Unauthorized - check your API token");
        throw new Error("Unauthorized - invalid or missing API token");
      } else if (response.status === 422) {
        ctx.logger.error(`Validation error: ${errorMessage}`);
        throw new Error(`Invalid PkgSpec: ${errorMessage}`);
      } else {
        ctx.logger.error(
          `Server error (${response.status}): ${errorMessage}`,
        );
        throw new Error(`Upload failed: ${errorMessage}`);
      }
    }

    const result: InstallFromFileV1Response = await response.json();

    ctx.logger.info("");
    ctx.logger.info("✓ Schema uploaded successfully!");
    ctx.logger.info("---");
    ctx.logger.info(`  Schema Name: ${result.schemaName}`);
    ctx.logger.info(`  Display Name: ${result.displayName}`);
    ctx.logger.info(`  Category: ${result.category}`);
    ctx.logger.info(`  Schema ID: ${result.schemaId}`);
    ctx.logger.info(`  Schema Variant ID: ${result.schemaVariantId}`);
    ctx.logger.info("");

    const changeSetUrl = `${instanceUrl}/w/${workspaceId}/${changeSetId}/l/a`;
    ctx.logger.info(`View in workspace: ${changeSetUrl}`);

    ctx.analytics.trackEvent("schema upload", {
      schemaId: result.schemaId,
      schemaName: result.schemaName,
      schemaVariantId: result.schemaVariantId,
    });
  } catch (error) {
    if (error instanceof Error && error.message.startsWith("Upload failed")) {
      throw error;
    }

    const errorMsg = unknownValueToErrorMessage(error);
    ctx.logger.error(`Failed to upload spec: ${errorMsg}`);
    throw error;
  }
}
