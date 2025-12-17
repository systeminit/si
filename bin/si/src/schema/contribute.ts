import { SchemasApi } from "@systeminit/api-client";
import { Context } from "../context.ts";
import { unknownValueToErrorMessage } from "../helpers.ts";

/**
 * Options for contributing a schema
 */
export interface SchemaContributeOptions {
  /**
   * Schema name or ID to contribute
   */
  schema: string;
}

/**
 * Contributes a schema to the module index
 *
 * This command works ONLY on HEAD change set and requires:
 * - The schema's default variant must be locked
 *
 * @param options - Options containing the schema identifier
 */
export async function callSchemaContribute(
  options: SchemaContributeOptions,
): Promise<void> {
  const ctx = Context.instance();
  const { schema } = options;

  ctx.logger.info(`Contributing schema: ${schema}`);
  ctx.logger.info("---");
  ctx.logger.info("");

  const apiConfig = Context.apiConfig();
  const workspaceId = Context.workspaceId();
  const schemasApi = new SchemasApi(apiConfig);

  try {
    // HEAD is always the change set we work with for contributions
    const changeSetId = "HEAD";

    ctx.logger.info("Looking up schema...");

    // First, try to find the schema by name or ID
    let schemaId: string;

    // Check if the input looks like an ID (UUID-like format)
    if (
      schema.match(
        /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i,
      ) ||
      schema.match(/^[0-9A-HJKMNP-TV-Z]{26}$/) // ULID format
    ) {
      // Treat as ID
      schemaId = schema;
      ctx.logger.info(`Using schema ID: ${schemaId}`);
    } else {
      // Treat as name - find the schema
      try {
        const { data: findResponse } = await schemasApi.findSchema({
          workspaceId,
          changeSetId,
          schema,
        });

        schemaId = findResponse.schemaId;
        ctx.logger.info(
          `Found schema: ${findResponse.schemaName} (${schemaId})`,
        );
      } catch (error) {
        ctx.logger.error(`Schema not found: ${schema}`);
        ctx.logger.error(
          "Please check the schema name or use the schema ID instead.",
        );
        throw error;
      }
    }

    ctx.logger.info("");
    ctx.logger.info("Contributing schema to module index...");

    // Call the contribute endpoint
    // Note: The SDK types say void, but the API actually returns {success: true, hash: string}
    const response = await schemasApi.contribute({
      workspaceId,
      changeSetId,
      schemaId,
    });

    // Check the response for success
    const responseData = response.data as unknown as {
      success?: boolean;
      hash?: string;
    };

    if (!responseData || responseData.success !== true) {
      throw new Error("Schema contribution failed: success flag not set");
    }

    ctx.logger.info("");
    ctx.logger.info("✓ Schema contributed successfully!");

    if (responseData.hash) {
      ctx.logger.info(`  Module hash: ${responseData.hash}`);
    }

    ctx.logger.info("");
    ctx.logger.info(
      "The contribution will be reviewed by the System Initiative staff for inclusion in the Module Index.",
    );

    ctx.analytics.trackEvent("schema contribute", {
      schemaId,
    });
  } catch (error) {
    const errorMsg = unknownValueToErrorMessage(error);

    ctx.logger.error("Failed to contribute schema");
    ctx.logger.error("---");

    // Handle known error cases with helpful messages
    if (errorMsg.includes("ContributionsMustBeMadeFromHead")) {
      ctx.logger.error(
        "ERROR: Contributions can only be made from the HEAD change set.",
      );
      ctx.logger.error(
        "This command automatically uses HEAD - this error should not occur.",
      );
    } else if (
      errorMsg.includes("ContributeUnlockedVariant") ||
      errorMsg.includes("unlocked schema variant")
    ) {
      ctx.logger.error(
        "ERROR: The schema's default variant must be locked before contributing.",
      );
      ctx.logger.error(
        "Please lock the schema variant in the System Initiative UI first.",
      );
    } else if (errorMsg.includes("404") || errorMsg.includes("not found")) {
      ctx.logger.error(`ERROR: Schema not found: ${schema}`);
      ctx.logger.error("Please check the schema name or ID and try again.");
    } else {
      ctx.logger.error(`ERROR: ${errorMsg}`);
    }

    throw error;
  }
}
