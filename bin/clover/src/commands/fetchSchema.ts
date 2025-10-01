import { CommandFailed } from "../errors.ts";
import _logger from "../logger.ts";
import { Provider } from "../types.ts";
import { PROVIDER_REGISTRY } from "../pipelines/types.ts";
import "../pipelines/aws/spec.ts";
import "../pipelines/hetzner/provider.ts";
import "../pipelines/dummy/spec.ts";

const logger = _logger.ns("fetchSchema").seal();

export async function fetchSchema(provider: Provider) {
  if (provider === "all") {
    for (const [name, config] of Object.entries(PROVIDER_REGISTRY)) {
      if (config.fetchSchema) {
        logger.info(`Fetching schema for provider: ${name}`);
        try {
          await config.fetchSchema();
          logger.info(`✓ Successfully fetched schema for ${name}`);
        } catch (err) {
          const message = err instanceof Error ? err.message : String(err);
          logger.error(`✗ Failed to fetch schema for ${name}: ${message}`);
          throw new CommandFailed(`Failed to fetch schema for ${name}`);
        }
      } else {
        logger.debug(`Skipping ${name} (no fetchSchema implementation)`);
      }
    }
    return;
  }

  const config = PROVIDER_REGISTRY[provider];
  if (!config) {
    throw new CommandFailed(`Unknown provider: ${provider}`);
  }

  if (!config.fetchSchema) {
    throw new CommandFailed(
      `Provider ${provider} does not support schema fetching`,
    );
  }

  try {
    logger.info(`Fetching schema for provider: ${provider}`);
    await config.fetchSchema();
    logger.info(`✓ Successfully fetched schema for ${provider}`);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    logger.error(`✗ Failed to fetch schema for ${provider}: ${message}`);
    throw new CommandFailed(`Failed to fetch schema for ${provider}`);
  }
}
