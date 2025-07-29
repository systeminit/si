// deno-lint-ignore-file no-explicit-any
import { getServiceByName, loadCfDatabase } from "@systeminit/cf-db";
import { logger } from "../logger.ts";
import { z } from "zod";

logger.debug("Loading AWS CloudFormation Database");
await loadCfDatabase({});

export const ServiceDocumentationSchemaRaw = {
  description: z.string().describe("A high level overview of the schema"),
  link: z.string().optional().describe(
    "A URL you can look up on the web to gain more information about the service",
  ),
};
export const ServiceDocumentationSchema = z.object(
  ServiceDocumentationSchemaRaw,
).describe("Documentation about a schema");
type ServiceDocumentation = z.infer<typeof ServiceDocumentationSchema>;

export function getDocumentationForService(
  serviceName: string,
): ServiceDocumentation {
  try {
    const cfSchema = getServiceByName(serviceName);
    const result: ServiceDocumentation = {
      description: cfSchema.description,
    };
    if (
      cfSchema.documentationUrl && typeof cfSchema.documentationUrl === "string"
    ) {
      result.link = cfSchema.documentationUrl;
    } else if (
      cfSchema.resourceLink && typeof cfSchema.resourceLink === "string"
    ) {
      result.link = cfSchema.resourceLink;
    }
    return result;
  } catch (_error) {
    return {
      description: `No documentation available for ${serviceName}`,
    };
  }
}

const AttributesForServiceSchema = z.array(
  z.object({
    name: z.string().describe("the attributes name"),
    path: z.string().describe(
      "the absolute path of the attribute, which you can use to look up its documentation",
    ),
    required: z.boolean().describe("if this attribute is required"),
  }).describe("an attribute"),
).describe("array of attributes");
type AttributesForService = z.infer<typeof AttributesForServiceSchema>;

export function getAttributesForService(
  serviceName: string,
): AttributesForService {
  try {
    const cfSchema = getServiceByName(serviceName);
    const attributes: AttributesForService = [];

    // Get required properties from schema
    const createOnlyProps = new Set(cfSchema.createOnlyProperties || []);
    const readOnlyProps = new Set(cfSchema.readOnlyProperties || []);
    const writeOnlyProps = new Set(cfSchema.writeOnlyProperties || []);

    // deno-lint-ignore no-inner-declarations
    function isRequired(path: string, localRequired: string[] = []): boolean {
      return createOnlyProps.has(path) ||
        readOnlyProps.has(path) ||
        writeOnlyProps.has(path) ||
        localRequired.includes(path.split("/").pop() || "");
    }

    // deno-lint-ignore no-inner-declarations
    function walkProperties(
      obj: any,
      basePath: string,
      requiredProps: string[] = [],
    ): void {
      if (!obj || typeof obj !== "object") return;

      for (const [key, value] of Object.entries(obj)) {
        if (!value || typeof value !== "object") continue;

        const currentPath = `${basePath}/${key}`;
        const prop = value as any;

        if (prop.type === "array") {
          // Handle arrays - use [array] in path for array elements
          const arrayPath = `${currentPath}/[array]`;

          if (prop.items?.type === "object" && prop.items.properties) {
            // Array of objects - recurse into object properties
            walkProperties(
              prop.items.properties,
              arrayPath,
              prop.items.required || [],
            );
          } else {
            // Simple array - add the array property itself
            attributes.push({
              name: key,
              path: currentPath,
              required: isRequired(currentPath, requiredProps),
            });
          }
        } else if (prop.type === "object") {
          if (
            prop.additionalProperties &&
            typeof prop.additionalProperties === "object"
          ) {
            // Handle maps - use '[map]' in path for map values
            const mapPath = `${currentPath}/[map]`;
            const mapValueType = prop.additionalProperties;

            if (mapValueType.type === "object" && mapValueType.properties) {
              walkProperties(
                mapValueType.properties,
                mapPath,
                mapValueType.required || [],
              );
            } else {
              attributes.push({
                name: key,
                path: currentPath,
                required: isRequired(currentPath, requiredProps),
              });
            }
          } else if (prop.properties) {
            // Regular nested object
            walkProperties(prop.properties, currentPath, prop.required || []);
          }
        } else {
          // Simple property (string, integer, boolean, etc.)
          attributes.push({
            name: key,
            path: currentPath,
            required: isRequired(currentPath, requiredProps),
          });
        }
      }
    }

    // Start walking from the root properties with /domain prefix
    if (cfSchema.properties) {
      walkProperties(cfSchema.properties, "/domain");
    }

    return attributes;
  } catch (error) {
    logger.error(`Failed to get attributes for service ${serviceName}:`, error);
    return [];
  }
}

export function getSchemaAttributeDocumentation(
  schemaName: string,
  schemaAttributePath: string,
): string | null {
  try {
    const cfSchema = getServiceByName(schemaName);

    // Remove /domain prefix from path for traversal
    const path = schemaAttributePath.startsWith("/domain/")
      ? schemaAttributePath.slice("/domain".length)
      : schemaAttributePath;

    // Split path into segments, filtering out empty strings
    const pathSegments = path.split("/").filter((segment) =>
      segment.length > 0
    );

    if (pathSegments.length === 0 || !cfSchema.properties) {
      return null;
    }

    // deno-lint-ignore no-inner-declarations
    function traversePath(
      obj: any,
      segments: string[],
      currentPath: string,
    ): string | null {
      if (!obj || typeof obj !== "object") return null;

      if (segments.length === 0) {
        return obj.description || null;
      }

      const [currentSegment, ...remainingSegments] = segments;
      const nextProperty = obj[currentSegment];

      if (!nextProperty) return null;

      // Handle array traversal
      if (currentSegment === "[array]") {
        // We're looking for documentation on array items
        if (remainingSegments.length === 0) {
          return obj.description || null;
        }

        if (obj.items && obj.items.properties) {
          return traversePath(
            obj.items.properties,
            remainingSegments,
            `${currentPath}/[array]`,
          );
        }
        return null;
      }

      // Handle map traversal
      if (currentSegment === "[map]") {
        if (remainingSegments.length === 0) {
          return obj.description || null;
        }

        if (obj.additionalProperties && obj.additionalProperties.properties) {
          return traversePath(
            obj.additionalProperties.properties,
            remainingSegments,
            `${currentPath}/[map]`,
          );
        }
        return null;
      }

      // Regular property traversal
      if (nextProperty.type === "array") {
        if (remainingSegments.length === 0) {
          return nextProperty.description || null;
        }

        // Check if next segment is [array]
        if (remainingSegments[0] === "[array]") {
          return traversePath(
            nextProperty,
            remainingSegments,
            `${currentPath}/${currentSegment}`,
          );
        }

        // If looking for a specific field within array items but no [array] specified,
        // fall back to the array's documentation
        return nextProperty.description || null;
      } else if (nextProperty.type === "object") {
        if (remainingSegments.length === 0) {
          return nextProperty.description || null;
        }

        // Handle maps (objects with additionalProperties)
        if (
          nextProperty.additionalProperties && remainingSegments[0] === "[map]"
        ) {
          return traversePath(
            nextProperty,
            remainingSegments,
            `${currentPath}/${currentSegment}`,
          );
        }

        // Regular nested object
        if (nextProperty.properties) {
          const result = traversePath(
            nextProperty.properties,
            remainingSegments,
            `${currentPath}/${currentSegment}`,
          );

          // If we couldn't find documentation for the specific field, fall back to the parent object
          if (result === null && nextProperty.description) {
            return nextProperty.description;
          }

          return result;
        }

        return nextProperty.description || null;
      } else {
        // Simple property
        if (remainingSegments.length === 0) {
          return nextProperty.description || null;
        }

        // Path continues but this is a simple property - invalid path
        return null;
      }
    }

    const result = traversePath(cfSchema.properties, pathSegments, "");

    // If we couldn't find specific documentation, try to find fallback documentation
    // by traversing up the path for array/map containers
    if (
      result === null &&
      (schemaAttributePath.includes("[array]") ||
        schemaAttributePath.includes("[map]"))
    ) {
      // Try to find the parent array or map documentation
      const parentPath = schemaAttributePath.split("/").slice(0, -1).join("/");
      if (parentPath !== schemaAttributePath) {
        return getSchemaAttributeDocumentation(schemaName, parentPath);
      }
    }

    return result;
  } catch (error) {
    logger.error(
      `Failed to get documentation for ${schemaName} path ${schemaAttributePath}:`,
      error,
    );
    return null;
  }
}
