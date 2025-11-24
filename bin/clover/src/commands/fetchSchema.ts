import { CommandFailed } from "../errors.ts";
import _logger from "../logger.ts";
import { FetchSchemaOptions, selectedProviders } from "../pipelines/types.ts";
import "../pipelines/aws/spec.ts";
import "../pipelines/hetzner/provider.ts";
import "../pipelines/azure/provider.ts";
import "../pipelines/entra/provider.ts";
import "../pipelines/dummy/spec.ts";

const logger = _logger.ns("fetchSchema").seal();

export async function fetchSchema(options: FetchSchemaOptions) {
  for (const provider of selectedProviders(options)) {
    try {
      logger.info(`Fetching schema for provider: ${provider.name}`);
      await provider.fetchSchema(options);
      logger.info(`✓ Successfully fetched schema for ${provider.name}`);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      logger.error(`✗ Failed to fetch schema for ${provider.name}: ${message}`);
      throw new CommandFailed(`Failed to fetch schema for ${provider.name}`);
    }
  }
}
