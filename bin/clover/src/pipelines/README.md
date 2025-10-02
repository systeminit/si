# Provider Pipelines

This directory contains the infrastructure for transforming provider-specific resource schemas into System Initiative's unified spec format.

## Architecture

The provider system is built around a centralized `ProviderConfig` pattern that encapsulates all provider-specific behavior:

- **Provider Registry**: A single source of truth mapping provider names to configurations (`PROVIDER_REGISTRY` in `types.ts`)
- **Generic Pipeline**: Shared logic for transforming schemas into specs (`generic/`)
- **Provider-Specific Pipelines**: Custom implementations for each provider (`aws/`, `hetzner/`, `dummy/`)

## Adding a New Provider

Follow these steps to add a new provider:

### 1. Create Provider Directory Structure

Create a new directory under `src/pipelines/` for your provider:

```
src/pipelines/your-provider/
├── provider.ts       # Provider configuration and hook implementations
├── schema.ts         # Provider-specific schema type definitions
├── spec.ts           # Schema transformation logic
├── funcs.ts          # Function spec definitions
├── pipeline.ts       # Pipeline orchestration
└── pipeline-steps/   # Optional: custom pipeline steps
```

### 2. Define Schema Types

In `schema.ts`, define your provider's schema types:

```typescript
import type { CfProperty, CfHandler, CfHandlerKind } from "../types.ts";

type JSONPointer = string;

export type YourProviderSchema = {
  typeName: string;
  description: string;
  properties: Record<string, CfProperty>;
  requiredProperties: Set<string>;
  primaryIdentifier: JSONPointer[];
  handlers?: Record<CfHandlerKind, CfHandler>;
  // ... provider-specific fields
};
```

### 3. Define Provider Functions

In `provider.ts`, implement the required provider functions:

```typescript
import {
  ProviderConfig,
  ProviderFunctions,
  ProviderFuncSpecs,
  SuperSchema,
  PROVIDER_REGISTRY,
} from "../types.ts";
import { normalizeOnlyProperties } from "../generic/index.ts";

/**
 * Create documentation links for schemas, definitions, and properties
 */
function createDocLink(
  schema: SuperSchema,
  defName: string | undefined,
  propName?: string,
): string {
  // Return a URL to your provider's documentation
  return `https://docs.yourprovider.com/...`;
}

/**
 * Determine the category for organizing resources
 */
function getCategory(schema: SuperSchema): string {
  // Extract category from schema.typeName
  // e.g., "YourProvider::Resource::Server" -> "YourProvider::Resource"
  return schema.typeName.split("::").slice(0, 2).join("::");
}

/**
 * Provider functions implementation
 */
const yourProviderFunctions: ProviderFunctions = {
  createDocLink,
  getCategory,
};
```

### 4. Define Function Specs

In `funcs.ts`, define your provider's action, code generation, management, and qualification functions:

```typescript
import { FuncSpecInfo } from "../../spec/funcs.ts";

export const ACTION_FUNC_SPECS = {
  "Create": {
    id: "unique-hash-here",
    displayName: "Create Resource",
    path: "./src/pipelines/your-provider/funcs/actions/create.ts",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "create",
  },
  "Refresh": {
    id: "unique-hash-here",
    displayName: "Refresh Resource",
    path: "./src/pipelines/your-provider/funcs/actions/refresh.ts",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "refresh",
  },
  // ... more actions (update, delete)
} as const satisfies Record<string, FuncSpecInfo>;

export const CODE_GENERATION_FUNC_SPECS = {
  "Generate Code": {
    id: "unique-hash-here",
    displayName: "Generate Code",
    path: "./src/pipelines/your-provider/funcs/codegen/generate.ts",
    backendKind: "jsAttribute",
    responseType: "codeGeneration",
  },
} as const satisfies Record<string, FuncSpecInfo>;

export const MANAGEMENT_FUNCS = {
  // Management functions (discover, import)
} as const satisfies Record<string, FuncSpecInfo>;

export const QUALIFICATION_FUNC_SPECS = {
  // Qualification functions
} as const satisfies Record<string, FuncSpecInfo>;
```

### 5. Complete Provider Configuration

Back in `provider.ts`, create the full provider configuration:

```typescript
function yourNormalizeProperty(
  prop: CfProperty,
  context: PropertyNormalizationContext,
): CfProperty {
  if ("properties" in prop && prop.properties && !prop.type) {
    return { ...prop, type: "object" } as CfProperty;
  }
  return prop;
}

function yourIsChildRequired(
  schema: SuperSchema,
  parentProp:
    | import("../../spec/props.ts").ExpandedPropSpecFor[
      "object" | "array" | "map"
    ]
    | undefined,
  childName: string,
): boolean {
  if ("requiredProperties" in schema) {
    return schema.requiredProperties.has(childName);
  }
  return false;
}

function yourClassifyProperties(schema: SuperSchema): OnlyProperties {
  return {
    createOnly: [],
    readOnly: ["id", "status"],
    writeOnly: [],
    primaryIdentifier: ["id"],
  };
}

async function yourLoadSchemas(options: import("../types.ts").PipelineOptions) {
  const { generateYourProviderSpecs } = await import("./pipeline.ts");
  return await generateYourProviderSpecs(options);
}

async function yourFetchSchema() {
  const url = "https://api.yourprovider.com/schema.json";
  const resp = await fetch(url);
  if (!resp.ok) {
    throw new Error(`Failed to fetch schema from ${url}`);
  }
  const schema = await resp.json();
  await Deno.writeTextFile(
    "./src/provider-schemas/yourprovider.json",
    JSON.stringify(schema, null, 2),
  );
}

const yourProviderFunctions: ProviderFunctions = {
  createDocLink,
  getCategory,
};

const yourProviderFuncSpecs: ProviderFuncSpecs = {
  actions: ACTION_FUNC_SPECS,
  codeGeneration: CODE_GENERATION_FUNC_SPECS,
  management: MANAGEMENT_FUNCS,
  qualification: QUALIFICATION_FUNC_SPECS,
};

export const yourProviderConfig: ProviderConfig = {
  name: "yourprovider",
  functions: yourProviderFunctions,
  funcSpecs: yourProviderFuncSpecs,
  loadSchemas: yourLoadSchemas,
  fetchSchema: yourFetchSchema,
  classifyProperties: yourClassifyProperties,
  metadata: {
    color: "#YOUR_HEX_COLOR",
    displayName: "Your Provider",
    description: "Your provider's resources",
  },
  normalizeProperty: yourNormalizeProperty,
  isChildRequired: yourIsChildRequired,
};

PROVIDER_REGISTRY[yourProviderConfig.name] = yourProviderConfig;
```

### 6. Implement Schema Transformation

In `spec.ts`, implement functions to transform your schemas:

```typescript
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { makeModule } from "../generic/index.ts";
import { yourProviderConfig } from "./provider.ts";
import type { YourProviderSchema } from "./schema.ts";

export function transformToSpec(schema: YourProviderSchema): ExpandedPkgSpec {
  const onlyProperties = yourProviderConfig.classifyProperties(schema);

  return makeModule(
    schema,
    schema.description,
    onlyProperties,
    yourProviderConfig,
  );
}
```

### 7. Implement Pipeline Orchestration

In `pipeline.ts`, implement the main pipeline function:

```typescript
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { PipelineOptions } from "../types.ts";
import { generateDefaultFuncsFromConfig } from "../generic/index.ts";
import { yourProviderConfig } from "./provider.ts";
import { transformToSpec } from "./spec.ts";

export async function generateYourProviderSpecs(
  options: PipelineOptions,
): Promise<ExpandedPkgSpec[]> {
  // 1. Load your provider's schemas from file or API
  const rawSchemas = await loadYourProviderSchemas();

  // 2. Transform schemas into specs
  let specs: ExpandedPkgSpec[] = rawSchemas.map(transformToSpec);

  // 3. Apply generic pipeline steps (adds all default functions)
  specs = generateDefaultFuncsFromConfig(specs, yourProviderConfig);

  // 4. Apply any custom pipeline steps (optional)
  specs = await customPipelineStep(specs);

  return specs;
}

async function loadYourProviderSchemas(): Promise<YourProviderSchema[]> {
  // Load from file, API, or generate mock schemas
  const data = await Deno.readTextFile("./src/provider-schemas/yourprovider.json");
  return JSON.parse(data);
}
```

### 8. Test Your Provider

Create a test file `pipeline.test.ts`:

```typescript
import { assertEquals } from "std/assert/mod.ts";
import { generateYourProviderSpecs } from "./pipeline.ts";

Deno.test("generateYourProviderSpecs - should generate specs", async () => {
  const specs = await generateYourProviderSpecs({
    forceUpdateExistingPackages: false,
    moduleIndexUrl: "...",
    docLinkCache: "...",
    inferred: "...",
  });

  assertEquals(specs.length > 0, true);
});
```

Run tests:

```bash
buck2 run //bin/clover:test-unit
```

Generate specs:

```bash
deno task run generate-specs --provider=yourprovider
```

Fetch schemas (if you implemented `fetchSchema`):

```bash
deno task run fetch-schema --provider=yourprovider
```

## File Structure

Each provider follows a consistent structure:

- **`schema.ts`** - Provider-specific schema type definitions
- **`provider.ts`** - Provider configuration, hooks, and registration
- **`spec.ts`** - Schema transformation logic (raw schema → ExpandedPkgSpec)
- **`funcs.ts`** - Function spec definitions (actions, code gen, management, qualification)
- **`pipeline.ts`** - Pipeline orchestration (loading, transforming, applying steps)
- **`pipeline-steps/`** - Custom transformation steps (optional)

## Key Concepts

### SuperSchema

A unified type that represents all provider schema formats:

```typescript
export type SuperSchema = HetznerSchema | CfSchema | YourProviderSchema;
```

All providers normalize their schemas to this union type.

### OnlyProperties

Classifies properties by their mutability:

- `createOnly`: Properties that can only be set during creation
- `readOnly`: Properties that are set by the provider (output-only)
- `writeOnly`: Properties that are input-only (like passwords)
- `primaryIdentifier`: Properties that uniquely identify the resource

### makeModule

The core function that transforms a schema into an `ExpandedPkgSpec`. It:

1. Splits properties into domain (mutable) and resource_value (read-only)
2. Creates prop specs for each property
3. Generates the schema variant with metadata (color, description, etc.)
4. Returns a complete module spec

### generateDefaultFuncsFromConfig

A helper that generates all default functions (actions, leaf, management, qualification) from your `ProviderConfig`. This eliminates boilerplate and ensures consistency.

## Examples

Study these providers to understand different implementation approaches:

- **Dummy** (`dummy/`): Minimal test provider - simplest implementation, great starting point
- **Hetzner** (`hetzner/`): OpenAPI-based provider with schema parsing and property inference
- **AWS** (`aws/`): CloudFormation-based provider with complex pipeline steps and extensive customization

## Common Patterns

### Schema Normalization Hooks

Every provider must implement `normalizeProperty` to transform provider-specific schemas into a format compatible with SI's property creation system:

**Common normalization tasks:**
- Infer missing types from structure
- Handle OpenAPI oneOf/anyOf composition (see `hetzner/provider.ts`)
- Apply CloudFormation normalization (see `aws/provider.ts`)
- Transform provider-specific property formats

### Custom Required Property Logic

Every provider must implement `isChildRequired` to determine which properties are required.

**Common approaches:**
- Check schema-level Set: `schema.requiredProperties.has(childName)` (OpenAPI/Hetzner)
- Check parent's array: `parentProp.cfProp.required?.includes(childName)` (CloudFormation/AWS)

### Pipeline Steps

Organize complex transformations into pipeline steps:

```typescript
specs = await addDefaultProps(specs);
specs = await addCustomBehavior(specs);
specs = await pruneUnwantedResources(specs);
```

### Property Classification Strategy

Classify properties as `createOnly`, `readOnly`, `writeOnly`, or `primaryIdentifier`:

**CloudFormation** (AWS): Explicitly marked in schema
```typescript
// See aws/provider.ts - awsClassifyProperties()
const onlyProperties: OnlyProperties = {
  createOnly: normalizeOnlyProperties(cfSchema.createOnlyProperties),
  readOnly: normalizeOnlyProperties(cfSchema.readOnlyProperties),
  writeOnly: normalizeOnlyProperties(cfSchema.writeOnlyProperties),
  primaryIdentifier: normalizeOnlyProperties(cfSchema.primaryIdentifier),
};
```

**OpenAPI** (Hetzner): Infer from HTTP methods
```typescript
// See hetzner/spec.ts - mergeResourceOperations()
const onlyProperties: OnlyProperties = {
  createOnly: [], // In POST but not PUT
  readOnly: [],   // In GET but not in POST/PUT/DELETE
  writeOnly: [],  // In POST/PUT/DELETE but not GET
  primaryIdentifier: ["id"],
};
```

## Questions?

Review existing providers for reference implementations:
1. **Start here**: `dummy/` - Simplest possible implementation
2. **OpenAPI providers**: `hetzner/` - Schema parsing and property inference
3. **Advanced customization**: `aws/` - Complex pipeline steps and overrides
