import _logger from "../../logger.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { makeModule, createPropFinder } from "./index.ts";
import { CfProperty, ProviderConfig } from "../types.ts";
import {
  createActionFuncs,
  createAttributeFuncs,
  createCodeGenFuncs,
  createManagementFuncs,
  createQualificationFuncs,
} from "./funcFactories.ts";
import {
  createActionFuncSpec,
  createLeafFuncSpec,
  createManagementFuncSpec,
} from "../../spec/funcs.ts";
import _ from "lodash";

const logger = _logger.ns("loadExtraAssets").seal();

/**
 * Pipeline step to load and merge manually-managed assets into the provider's specs.
 *
 * This step:
 * 1. Checks if the provider has extraAssets configured
 * 2. Loads the manually-managed asset schemas
 * 3. Transforms them into ExpandedPkgSpecs using the same pipeline as auto-generated assets
 * 4. Applies custom functions if specified (otherwise uses provider defaults)
 * 5. Merges them with the existing specs
 *
 * Manually-managed assets are useful for:
 * - Adding custom resources that don't exist in the provider's API specs
 * - Resources you want full manual control over
 * - Extending providers without modifying their auto-generation logic
 *
 * Custom functions:
 * - If customFuncs are defined for a manually-managed asset, ONLY those functions are used
 * - Default provider functions are NOT added if custom functions are specified
 * - This gives you complete control over the function implementation
 *
 * @param incomingSpecs - The specs from the auto-generated pipeline
 * @param providerConfig - The provider configuration
 * @returns Combined array of auto-generated and manually-managed asset specs
 */
export async function loadExtraAssets(
  incomingSpecs: ExpandedPkgSpec[],
  providerConfig: ProviderConfig,
): Promise<ExpandedPkgSpec[]> {
  // If no manually-managed assets configured, return the incoming specs unchanged
  if (!providerConfig.extraAssets) {
    return incomingSpecs;
  }

  logger.info(
    `Loading manually-managed assets for provider: ${providerConfig.name}`,
  );

  // Load the manually-managed asset schemas
  const extraSchemas = await providerConfig.extraAssets.loadSchemas();

  if (extraSchemas.length === 0) {
    logger.info(
      `No manually-managed assets found for provider: ${providerConfig.name}`,
    );
    return incomingSpecs;
  }

  logger.info(
    `Found ${extraSchemas.length} manually-managed asset(s) for provider: ${providerConfig.name}`,
  );

  // Transform each manually-managed schema into an ExpandedPkgSpec
  const extraSpecs: ExpandedPkgSpec[] = [];

  for (const schema of extraSchemas) {
    logger.debug(`Processing manually-managed asset: ${schema.typeName}`);

    // Use the manually-managed assets' classifyProperties function, or fall back to the provider's
    const classifyFn = providerConfig.extraAssets.classifyProperties ||
      providerConfig.classifyProperties;

    if (!classifyFn) {
      throw new Error(
        `No classifyProperties function found for manually-managed asset ${schema.typeName}. ` +
          `Provide either extraAssets.classifyProperties or provider.classifyProperties`,
      );
    }

    const onlyProperties = classifyFn(schema);

    // Split properties into domain (writable), resource_value (read-only), and secrets
    const readOnlySet = new Set(onlyProperties.readOnly);
    const writeOnlySet = new Set(onlyProperties.writeOnly);
    // Get explicit secret kinds mapping from schema (if any)
    const secretKindsMap = (schema as any).secretKinds || {};
    const domainProperties: Record<string, CfProperty> = {};
    const resourceValueProperties: Record<string, CfProperty> = {};
    const secretProperties: Record<string, CfProperty> = {};

    if ("properties" in schema) {
      for (const [name, prop] of Object.entries(schema.properties)) {
        if (readOnlySet.has(name)) {
          resourceValueProperties[name] = prop as CfProperty;
        } else if (secretKindsMap[name]) {
          // Only properties explicitly mapped in secretKinds go into secrets
          secretProperties[name] = prop as CfProperty;
        } else {
          // All other properties (including writeOnly without secretKind) go to domain
          domainProperties[name] = prop as CfProperty;
        }
      }
    }

    // Create the module spec using the generic makeModule helper
    // Pass secretProperties so they're created in the secrets section from the start
    const spec = makeModule(
      schema,
      schema.description,
      onlyProperties,
      providerConfig,
      domainProperties,
      resourceValueProperties,
      secretProperties,
    );

    // Get the variant to work with
    const variant = spec.schemas[0].variants[0];

    if (Object.keys(secretProperties).length > 0) {
      logger.info(
        `Created ${Object.keys(secretProperties).length} secret property(ies) for ${schema.typeName}: ${
          Object.keys(secretProperties).join(", ")
        }`,
      );

      // Auto-configure secret properties with appropriate secretKind
      // Uses explicit secretKinds mapping from the schema
      for (const secretName of Object.keys(secretProperties)) {
        const secretProp = variant.secrets.entries.find((p) => p.name === secretName);
        if (secretProp) {
          secretProp.data.widgetKind = "Secret";

          // Get the secretKind from the schema's explicit mapping
          const secretKind = (schema as any).secretKinds?.[secretName];

          if (!secretKind) {
            logger.warn(
              `No secretKind specified for writeOnly property "${secretName}" in ${schema.typeName}. ` +
              `Please add it to the schema's secretKinds mapping.`
            );
            continue;
          }

          secretProp.data.widgetOptions = [
            {
              label: "secretKind",
              value: secretKind,
            },
          ];

          logger.debug(`Configured secret "${secretName}" with secretKind: "${secretKind}"`);
        }
      }
    }

    // Check if this manually-managed asset has custom configuration defined
    const customConfig = providerConfig.extraAssets.customFuncs?.[
      schema.typeName
    ];

    // Apply custom metadata if provided
    if (customConfig?.metadata) {
      const schemaData = spec.schemas[0].data;

      logger.debug(
        `Applying custom metadata for manually-managed asset: ${schema.typeName}`,
      );

      if (customConfig.metadata.displayName !== undefined) {
        variant.data.displayName = customConfig.metadata.displayName;
      }
      if (customConfig.metadata.color !== undefined) {
        variant.data.color = customConfig.metadata.color;
      }
      if (customConfig.metadata.category !== undefined) {
        schemaData.category = customConfig.metadata.category;
      }
      if (customConfig.metadata.description !== undefined) {
        variant.data.description = customConfig.metadata.description;
      }
    }

    // Check if this manually-managed asset has custom functions defined
    const customFuncs = customConfig;

    if (customFuncs) {
      logger.debug(
        `Applying custom functions for manually-managed asset: ${schema.typeName}`,
      );

      // Get the domain ID for attaching functions
      const domainId = variant.domain.uniqueId;

      if (!domainId) {
        throw new Error(
          `Domain uniqueId not found for ${schema.typeName}. Cannot attach custom functions.`,
        );
      }

      // Apply custom action functions (if provided)
      if (customFuncs.actions) {
        const actionFuncs = createActionFuncs(customFuncs.actions);
        for (const { spec: actionFunc, kind } of actionFuncs) {
          spec.funcs.push(_.cloneDeep(actionFunc));
          variant.actionFuncs.push(
            createActionFuncSpec(kind, actionFunc.uniqueId),
          );
        }
      }

      // Apply custom code generation functions (if provided)
      if (customFuncs.codeGeneration) {
        const codeGenFuncs = createCodeGenFuncs(
          customFuncs.codeGeneration,
          domainId,
        );
        for (const codeGenFunc of codeGenFuncs) {
          spec.funcs.push(_.cloneDeep(codeGenFunc));
          variant.leafFunctions.push(
            createLeafFuncSpec(
              "codeGeneration",
              codeGenFunc.uniqueId,
              ["domain"],
            ),
          );
        }
      }

      // Apply custom management functions (if provided)
      if (customFuncs.management) {
        const mgmtFuncs = createManagementFuncs(customFuncs.management);
        for (const { func, handlers } of mgmtFuncs) {
          spec.funcs.push(_.cloneDeep(func));
          variant.managementFuncs.push(
            createManagementFuncSpec(func.name, func.uniqueId),
          );
        }
      }

      // Apply custom qualification functions (if provided)
      if (customFuncs.qualification) {
        const qualificationFuncs = createQualificationFuncs(
          customFuncs.qualification,
          domainId,
        );
        for (const qualificationFunc of qualificationFuncs) {
          spec.funcs.push(_.cloneDeep(qualificationFunc));
          variant.leafFunctions.push(
            createLeafFuncSpec(
              "qualification",
              qualificationFunc.uniqueId,
              ["domain", "code", "secrets" as any], // secrets is valid but not in the type definition
            ),
          );
        }
      }

      // Apply custom attribute functions (if provided)
      // Attribute functions compute property values and should be attached to those properties
      if (customFuncs.attribute) {
        if (!customFuncs.attributeFunctions) {
          logger.warn(
            `Attribute functions defined but no attributeFunctions configuration provided for ${schema.typeName}. ` +
              `Functions will not be attached to any properties.`,
          );
          continue;
        }

        logger.debug(
          `Applying attribute functions for: ${schema.typeName}`,
        );

        // Get the attribute function configuration
        const attributeConfig = customFuncs.attributeFunctions(variant);

        // Use the generic property finder
        const findProp = createPropFinder(variant, schema.typeName);

        // Process each attribute function
        for (const [funcName, config] of Object.entries(attributeConfig)) {
          const funcSpec = customFuncs.attribute[funcName];
          if (!funcSpec) {
            throw new Error(
              `Attribute function "${funcName}" configured but not defined in attribute map for ${schema.typeName}`,
            );
          }

          // Build function argument bindings from the input property names
          const funcBindings = config.inputs.map((propName) => {
            const prop: any = findProp(propName);
            return {
              name: propName,
              kind: prop.kind,
              elementKind: prop.elementKind || null,
              uniqueId: prop.uniqueId,
              deleted: false,
            };
          });

          // Create the function spec with bindings
          const attributeFuncs = createAttributeFuncs(
            { [funcName]: funcSpec },
            { [funcName]: funcBindings },
          );

          for (const attributeFunc of attributeFuncs) {
            spec.funcs.push(_.cloneDeep(attributeFunc));
          }

          // Attach the function to the specified property
          // Note: attachTo should be a top-level property path, not a nested array element property
          // Valid: "Tags", "Config.Timeout"
          // Invalid: "Tags.Key" (can't attach to array element property schema)
          const targetProp: any = findProp(config.attachTo);

          // Validate that we're not trying to attach to an array element property
          // (which would be attaching to the schema/type, not an actual instance)
          if (config.attachTo.includes(".")) {
            const parts = config.attachTo.split(".");
            // Check if any part of the path (except the last) is an array
            let checkProp: any = variant.domain.entries.find((p) =>
              p.name === parts[0]
            );
            for (let i = 1; i < parts.length; i++) {
              if (!checkProp) break;

              // If the current property is an array and we're going deeper, that's a problem
              if (checkProp.kind === "array" && i < parts.length) {
                throw new Error(
                  `Cannot attach function to "${config.attachTo}" in ${schema.typeName}. ` +
                    `You cannot attach functions to properties inside array elements (like "Tags.Key"). ` +
                    `Instead, attach to the array itself ("Tags") or to a separate property that processes the array.`,
                );
              }

              // Navigate to next level
              if (checkProp.kind === "object" && checkProp.entries) {
                checkProp = checkProp.entries.find((e: any) =>
                  e.name === parts[i]
                );
              }
            }
          }

          targetProp.data.funcUniqueId = funcSpec.id;

          // Set up property inputs with correct prop_paths
          targetProp.data.inputs = config.inputs.map((propPath) => {
            const prop: any = findProp(propPath);
            // For nested paths like "Config.MaxRetries", convert dots to the separator
            // "Config.MaxRetries" becomes "root\x0Bdomain\x0BConfig\x0BMaxRetries"
            const pathParts = propPath.split(".");
            const fullPath = ["root", "domain", ...pathParts].join("\x0B");

            return {
              kind: "prop" as const,
              name: propPath,
              prop_path: fullPath,
              unique_id: prop.uniqueId,
              deleted: false,
            };
          });

          logger.debug(
            `Attached attribute function "${funcName}" to property "${config.attachTo}" with inputs: ${
              config.inputs.join(", ")
            }`,
          );
        }
      }

      logger.info(
        `Applied custom functions for manually-managed asset: ${schema.typeName}`,
      );
    } else {
      logger.debug(
        `No custom functions defined for ${schema.typeName}, will use provider defaults`,
      );
      // Note: Default functions will be added by generateDefaultFuncsFromConfig
      // in the main pipeline, which runs AFTER loadExtraAssets
    }

    // Call configureProperties if provided
    if (customFuncs?.configureProperties) {
      logger.debug(
        `Configuring properties for manually-managed asset: ${schema.typeName}`,
      );
      customFuncs.configureProperties(variant);
      logger.info(
        `Configured properties for manually-managed asset: ${schema.typeName}`,
      );
    }

    extraSpecs.push(spec);
  }

  logger.info(
    `Successfully loaded ${extraSpecs.length} manually-managed asset(s) for provider: ${providerConfig.name}`,
  );

  // Merge manually-managed specs with incoming specs
  // Manually-managed specs come after auto-generated specs so they can be easily identified
  return [...incomingSpecs, ...extraSpecs];
}
