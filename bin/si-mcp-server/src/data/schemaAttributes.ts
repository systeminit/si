import { GetSchemaVariantV1Response } from "@systeminit/api-client";

// ---------- Your existing types ----------
type PropNode = {
  propId: string;
  name: string;
  propType: string;
  description: string | null;
  children: PropNode[] | null;
  validationFormat?: unknown;
  defaultValue?: unknown;
  hidden?: boolean;
  docLink?: string | null;
};

type FlatAttribute = { name: string; path: string; required: boolean };
type FlatSchema = { schemaName: string; attributes: FlatAttribute[] };

type DocMeta = { description: string | null; docLink: string | null };
export type DocsIndex = Map<string, DocMeta>;

type ValidationFlags = { presence?: string };
type ValidationFormat = { flags?: ValidationFlags } & Record<string, unknown>;

/** Best-effort parse for Joi-ish validationFormat (string or object). */
function toValidationFormat(input: unknown): ValidationFormat | null {
  if (!input) return null;

  if (typeof input === "string") {
    try {
      const parsed = JSON.parse(input) as unknown;
      return typeof parsed === "object" && parsed !== null
        ? (parsed as ValidationFormat)
        : null;
    } catch {
      return null;
    }
  }

  if (typeof input === "object") {
    return input as ValidationFormat;
  }

  return null;
}

/** Determines if a node is required based on validationFormat.flags.presence. */
function isRequiredFromValidationFormat(input: unknown): boolean {
  const vf = toValidationFormat(input);
  return vf?.flags?.presence === "required";
}

// ---------- collection helpers ----------
function addLeaf(
  outAttrs: FlatAttribute[],
  outDocs: DocsIndex | null,
  node: {
    name: string;
    validationFormat?: unknown;
    description: string | null;
    docLink?: string | null;
  },
  path: string,
): void {
  outAttrs.push({
    name: node.name,
    path,
    required: isRequiredFromValidationFormat(node.validationFormat),
  });
  if (outDocs) {
    outDocs.set(path, {
      description: node.description ?? null,
      docLink: node.docLink ?? null,
    });
  }
}

/**
 * Flattens domain properties into a list of {name, path, required}.
 * Optionally also builds a docs index { path -> {description, docLink} } in the same pass.
 */
function collectFlatAttributes(
  nodes: PropNode[] | null | undefined,
  basePath: string,
  outAttrs: FlatAttribute[],
  outDocs: DocsIndex | null, // pass a Map to collect docs, or null to skip
): void {
  if (!nodes || nodes.length === 0) return;

  for (const node of nodes) {
    const nodePath = `${basePath}/${node.name}`;

    switch (node.propType) {
      case "object": {
        const hasChildren = Array.isArray(node.children) &&
          node.children.length > 0;
        if (hasChildren) {
          collectFlatAttributes(node.children!, nodePath, outAttrs, outDocs);
        } else {
          addLeaf(outAttrs, outDocs, node, nodePath);
        }
        break;
      }

      case "array": {
        const arrayPath = `${nodePath}/[array]`;
        const hasChildren = Array.isArray(node.children) &&
          node.children.length > 0;

        if (!hasChildren) {
          addLeaf(outAttrs, outDocs, node, arrayPath);
          break;
        }

        for (const item of node.children!) {
          if (item.propType === "object") {
            const itemHasChildren = Array.isArray(item.children) &&
              item.children.length > 0;
            if (itemHasChildren) {
              collectFlatAttributes(
                item.children!,
                arrayPath,
                outAttrs,
                outDocs,
              );
            } else {
              addLeaf(outAttrs, outDocs, item, arrayPath);
            }
          } else {
            addLeaf(outAttrs, outDocs, item, `${arrayPath}/${item.name}`);
          }
        }
        break;
      }

      default: {
        addLeaf(outAttrs, outDocs, node, nodePath);
      }
    }
  }
}

export function buildAttributesStructure(
  input: GetSchemaVariantV1Response,
): FlatSchema {
  const attributes: FlatAttribute[] = [];
  const root = input.domainProps;
  if (!root) {
    // We can't build the attributesStructure so we return it empty!
    return {
      schemaName: input.displayName,
      attributes,
    };
  }
  const basePath = `/${root.name}`;
  collectFlatAttributes(
    root.children,
    basePath,
    attributes,
    /* outDocs */ null,
  );
  return { schemaName: input.displayName, attributes };
}

/** Build a docs index keyed by the same flat paths. */
export function buildAttributeDocsIndex(
  input: GetSchemaVariantV1Response,
): DocsIndex {
  const attributesSink: FlatAttribute[] = []; // unused but cheap
  const docsIndex: DocsIndex = new Map();
  const root = input.domainProps;
  if (!root) {
    // We can't build the docIndex so we return it empty!
    return docsIndex;
  }
  const basePath = `/${root.name}`;
  collectFlatAttributes(root.children, basePath, attributesSink, docsIndex);
  return docsIndex;
}

/** Turn a docs index + path into a final documentation string. */
export function formatDocumentation(
  docsIndex: DocsIndex,
  path: string,
): string | undefined {
  const meta = docsIndex.get(path);
  if (!meta) return undefined;

  const parts: string[] = [];
  if (meta.description) parts.push(meta.description.trim());
  if (meta.docLink) parts.push(`Documentation: ${meta.docLink}`);
  return parts.length > 0 ? parts.join("\n\n") : undefined;
}

export type SchemaAttributeDocumentation = {
  schemaAttributePath: string;
  documentation: string;
};

export type SchemaDocumentationData = {
  schemaName: string;
  attributes: SchemaAttributeDocumentation[];
};

/**
 * Builds documentation items for the given attribute paths against a single schema variant.
 * Produces data in the grouped shape expected by your MCP output schema.
 */
export function buildDocumentationForPaths(
  schemaName: string,
  variant: GetSchemaVariantV1Response,
  schemaAttributePaths: string[],
): SchemaDocumentationData {
  const docsIndex = buildAttributeDocsIndex(variant);
  const attributes = schemaAttributePaths.map((p) => {
    const documentation = formatDocumentation(docsIndex, p) ??
      "There is no documentation for this attribute; if it is an AWS schema, consider looking up the data for the corresponding cloudformation resource";
    return { schemaAttributePath: p, documentation };
  });

  return { schemaName, attributes };
}
