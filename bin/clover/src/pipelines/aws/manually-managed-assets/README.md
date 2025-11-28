# AWS Manually-Managed Assets - Complete Guide

## What Are Manually-Managed Assets?

Manually-managed assets are resources that you define from scratch instead of
auto-generating from CloudFormation specs. Use them for:

- AWS resources without CloudFormation support (e.g., Control Tower, WAFv2)
- Custom abstractions over AWS services
- Resources requiring specific custom behavior
- Testing new resource types before official support

## What You Need to Build

For each manually-managed asset, you create **three things**:

### 1. Schema Definition

Defines the resource properties (inputs and outputs).

### 2. Function Specifications

Declares what functions the resource has (actions, validations, queries).

### 3. Function Implementations

The actual TypeScript code that executes.

## Directory Layout

Each resource follows this structure:

```
manually-managed-assets/
└── AWS::{Service}::{Resource}/     # One directory per resource
    ├── schema.ts                    # Schema + function specs (REQUIRED)
    ├── actions/                     # Action implementations (optional)
    │   ├── create.ts
    │   ├── update.ts
    │   ├── delete.ts
    │   └── refresh.ts
    ├── qualifications/              # Validation implementations (optional)
    │   └── validate*.ts
    ├── attributes/                  # Query/compute implementations (optional)
    │   └── query*.ts
    ├── management/                  # Discovery implementations (optional)
    │   ├── discover.ts
    │   └── import.ts
    └── codegen/                     # Code generation implementations (optional)
        └── generate*.ts
```

## Step-by-Step: Creating a Manually-Managed Asset

### Step 1: Create the Directory

```bash
cd src/pipelines/aws/manually-managed-assets
mkdir -p "AWS::Service::Resource"
cd "AWS::Service::Resource"
```

### Step 2: Create `schema.ts`

This file contains **everything about your resource**. It has three sections:

#### Section A: Schema (REQUIRED)

Define your resource properties:

```typescript
import { CfSchema } from "../../../../schema.ts";
import { FuncSpecInfo } from "../../../../../../spec/funcs.ts";
import { ExpandedSchemaVariantSpec } from "../../../../../../spec/pkgs.ts";

export const schema: CfSchema = {
  typeName: "AWS::Service::Resource",
  description: "What this resource does",
  properties: {
    // USER-CONFIGURABLE PROPERTIES (domain)
    Name: {
      type: "string",
      description: "Resource name",
      default: "my-resource",  // Optional: set default values
      docLink: "https://docs.aws.amazon.com/...",  // Optional: custom doc link
    } as any,

    Configuration: {
      type: "object",
      description: "Configuration settings",
      properties: {
        // Nested properties
      },
    },

    Tags: {
      type: "array",
      description: "Resource tags",
      itemName: "Tag",  // Optional: override default "TagsItem" to prevent breaking changes
      items: {
        type: "object",
        properties: {
          Key: { type: "string" },
          Value: { type: "string" },
        },
      },
    } as any,

    // READ-ONLY PROPERTIES (resource_value)
    Id: {
      type: "string",
      description: "Resource ID assigned by AWS",
    },
    Arn: {
      type: "string",
      description: "Resource ARN",
    },

    // SECRET PROPERTIES
    credential: {
      type: "string",
      description: "AWS credential for authentication",
    },
  },
  definitions: {},
  primaryIdentifier: ["/properties/Id"],
  readOnlyProperties: ["/properties/Id", "/properties/Arn"],
  createOnlyProperties: ["/properties/Name"], // Can't change after creation
  writeOnlyProperties: [],  // Not used for secrets - use secretKinds instead
  handlers: {},

  // SECRETS: Explicit mapping of properties to secret kinds
  secretKinds: {
    credential: "AWS Credential",  // Properties here become secrets
  },
};
```

**Schema Customization Fields (all optional):**

- `default: value` - Set default values for properties (extracted to `.setDefaultValue()`)
- `docLink: "url"` - Override provider's default doc link (extracted to `.setDocLink()`)
- `itemName: "Name"` - Override array item name (default is `{PropertyName}Item`)
- `secretKinds: { propName: "Secret Kind" }` - Declare which properties are secrets

**Important Notes:**

1. **Secrets**: Don't use `writeOnlyProperties` for secrets. Use `secretKinds` instead:
   ```typescript
   // ❌ WRONG - writeOnly doesn't specify the secret kind
   writeOnlyProperties: ["/properties/credential"],

   // ✅ CORRECT - explicit secret kind mapping
   secretKinds: {
     credential: "AWS Credential",
   }
   ```

2. **Array Item Names**: Always specify `itemName` to prevent breaking changes:
   ```typescript
   // Without itemName, generates "FiltersItem" which may break user configs
   // With itemName, uses "Filter" to maintain compatibility
   Filters: {
     type: "array",
     itemName: "Filter",  // Preserves backward compatibility
     items: { ... }
   }
   ```

3. **Custom Fields**: Use `as any` type assertion for custom fields (`docLink`, `itemName`)

#### Section B: Schema Configuration (REQUIRED)

Declare functions, bindings, and metadata for your resource. Use
`uuidgen | shasum -a 256` to generate stable function IDs.

```typescript
export const config = {
  // ACTIONS: CRUD operations (optional - include what you need)
  actions: {
    Create: {
      id: "GENERATE_UNIQUE_ID_HERE", // Run: uuidgen | shasum -a 256
      displayName: "Create Resource",
      path: "./src/pipelines/aws/manually-managed-assets/AWS::Service::Resource/actions/create.ts",
      backendKind: "jsAction",
      responseType: "action",
      actionKind: "create",
    },
    Update: {
      id: "GENERATE_UNIQUE_ID_HERE",
      displayName: "Update Resource",
      path: "./src/pipelines/aws/manually-managed-assets/AWS::Service::Resource/actions/update.ts",
      backendKind: "jsAction",
      responseType: "action",
      actionKind: "update",
    },
    Delete: {
      id: "GENERATE_UNIQUE_ID_HERE",
      displayName: "Delete Resource",
      path: "./src/pipelines/aws/manually-managed-assets/AWS::Service::Resource/actions/delete.ts",
      backendKind: "jsAction",
      responseType: "action",
      actionKind: "delete",
    },
    Refresh: {
      id: "GENERATE_UNIQUE_ID_HERE",
      displayName: "Refresh Resource State",
      path: "./src/pipelines/aws/manually-managed-assets/AWS::Service::Resource/actions/refresh.ts",
      backendKind: "jsAction",
      responseType: "action",
      actionKind: "refresh",
    },
  } as const satisfies Record<string, FuncSpecInfo & { actionKind: string }>,

  // QUALIFICATIONS: Validations (optional - include if needed)
  qualification: {
    "Validate Configuration": {
      id: "GENERATE_UNIQUE_ID_HERE",
      displayName: "Validate Configuration",
      path: "./src/pipelines/aws/manually-managed-assets/AWS::Service::Resource/qualifications/validate.ts",
      backendKind: "jsAttribute",
      responseType: "qualification",
    },
  } as const satisfies Record<string, FuncSpecInfo>,

  // ATTRIBUTES: Query/compute values (optional - include if needed)
  attribute: {
    "Query Value": {
      id: "GENERATE_UNIQUE_ID_HERE",
      displayName: "Query Value",
      path: "./src/pipelines/aws/manually-managed-assets/AWS::Service::Resource/attributes/query.ts",
      backendKind: "jsAttribute",
      responseType: "string", // Use the return type: string, boolean, integer, json, etc.
    },
  } as const satisfies Record<string, FuncSpecInfo>,

  // MANAGEMENT: Discovery/import (optional - include if needed)
  management: {
    Discover: {
      id: "GENERATE_UNIQUE_ID_HERE",
      displayName: "Discover Resources",
      path: "./src/pipelines/aws/manually-managed-assets/AWS::Service::Resource/management/discover.ts",
      backendKind: "management",
      responseType: "management",
      handlers: ["list", "read"],
    },
  } as const satisfies Record<string, FuncSpecInfo & { handlers: string[] }>,

  // CODE GENERATION: Generate IaC (optional - include if needed)
  codeGeneration: {
    "Generate Terraform": {
      id: "GENERATE_UNIQUE_ID_HERE",
      displayName: "Generate Terraform",
      path: "./src/pipelines/aws/manually-managed-assets/AWS::Service::Resource/codegen/terraform.ts",
      backendKind: "jsAttribute",
      responseType: "codeGeneration",
    },
  } as const satisfies Record<string, FuncSpecInfo>,
};

export default { schema, config };
```

#### Section C: Attribute Function Configuration (Optional)

Only needed if you have attribute functions. This new simplified API automatically handles bindings:

```typescript
// Inside the config object:
attributeFunctions: (variant: ExpandedSchemaVariantSpec) => {
  return {
    "Query Value": {
      attachTo: "ComputedProperty",  // Property to attach function to
      inputs: ["region", "Name", "Config"],  // Domain properties to pass as inputs
    },
    "Another Attribute": {
      attachTo: "OtherProperty",
      inputs: ["Name"],
    },
  };
},
```

**The system automatically:**
- Looks up property types and uniqueIds
- Creates function argument bindings
- Attaches the function to the specified property
- Sets up property inputs with correct paths

**No more manual `attributeBindings`!** The old verbose approach is no longer needed.

#### Section D: Property Configuration (Optional)

Use `configureProperties` for advanced property configuration that can't be expressed in the schema:

```typescript
import { addPropSuggestSource, addPropSuggestAsSourceFor } from "../../../../spec/props.ts";
import { createPropFinder } from "../../../generic/index.ts";

// Inside the config object:
configureProperties: (variant: ExpandedSchemaVariantSpec) => {
  const findProp = createPropFinder(variant, "AWS::Service::Resource");

  // Set up property suggestions (for UI auto-completion)
  const regionProp = findProp("region");
  addPropSuggestSource(regionProp, {
    schema: "Region",
    prop: "/domain/region",
  });

  const idProp = findProp("Id");
  addPropSuggestAsSourceFor(idProp, {
    schema: "AWS::EC2::Instance",
    prop: "/domain/ResourceId",
  });

  // Note: Most configuration (docLinks, defaults, secretKinds) should be in the schema.
  // Only use configureProperties for things that can't be expressed declaratively.
},
```

#### Section E: Metadata (Optional)

Custom metadata to override provider defaults:

```typescript
// Inside the config object:
metadata: {
  displayName: "Custom Display Name",  // Override default name (null by default)
  category: "AWS::Custom::Category",   // Override auto-derived category
  color: "#FF6600",                    // Override provider default color
  description: "Custom description",   // Override schema.description if needed
},
```

All metadata fields are optional. If not provided, defaults are used:

- `displayName`: `null` (uses schema name)
- `category`: Derived from `typeName` (e.g., "AWS::EC2::AMI" → "AWS::EC2")
- `color`: Provider default (AWS: `#FF9900`)
- `description`: From `schema.description`

### Step 3: Implement Functions

Create the function files you declared. Each function type has a specific
signature. 

### Step 4: Register the Asset

Add your resource to `extraAssets.ts`:

```typescript
// In src/pipelines/aws/extraAssets.ts
import myResource from "./manually-managed-assets/AWS::Service::Resource/schema.ts";

export const AWS_EXTRA_ASSETS = {
  "AWS::EC2::AMI": ec2Ami,
  "AWS::Service::Resource": myResource, // ← Add this line
};
```

### Step 5: Generate and Test

```bash
# Generate the spec
deno task run generate-specs --provider=aws AWS::Service::Resource

# Check the output
cat si-specs/AWS::Service::Resource.json
```

## Function IDs: Critical Requirements

Each function MUST have a unique ID that:

- Is globally unique (across ALL functions in the system)
- Never changes once set (SI tracks functions by ID)
- Is a 64-character hex string (SHA256 hash)

### Generate a Function ID

```bash
uuidgen | shasum -a 256 | awk '{print $1}'
```

Example output:
`e8b9a8a41fd88e1a8cc3a1f3646af0a6dd4f7f578e0a7960b167cba28f5c4f4b`

### Store IDs Clearly

```typescript
const FUNC_IDS = {
  CREATE: "e8b9a8a41fd88e1a8cc3a1f3646af0a6dd4f7f578e0a7960b167cba28f5c4f4b",
  UPDATE: "f9c0b9b52ge89f2b9dd4b2g4757bg1b7ee5g8g679f1b8071c278dcb39g6d5g5c",
  DELETE: "a7d8c7c30hd77d0c8bb2a0f2535af0a5cc3e6e467d0a6850a056abc17e4b3e3a",
} as const;

export const funcs = {
  actions: {
    "Create": { id: FUNC_IDS.CREATE, ... },
    "Update": { id: FUNC_IDS.UPDATE, ... },
    "Delete": { id: FUNC_IDS.DELETE, ... },
  },
};
```

## Complete Working Example

See `AWS::EC2::AMI/` for a complete, working implementation that demonstrates:

- Schema with domain and resource_value properties
- Qualification function for validation
- Attribute function with custom bindings
- All three files working together

Study that example to understand the complete pattern.

## Quick Reference

| What                       | Where                   | Required?                    |
| -------------------------- | ----------------------- | ---------------------------- |
| Schema definition          | `schema.ts` (section A) | Yes                          |
| Function specs             | `schema.ts` (section B) | Yes (if using functions)     |
| Attribute configuration    | `schema.ts` (section C) | Only if using attributes     |
| Property configuration     | `schema.ts` (section D) | Only for suggestSource/etc.  |
| Metadata                   | `schema.ts` (section E) | Optional                     |
| Function implementations   | `{type}/*.ts` files     | Yes (for declared functions) |
| Registration               | `extraAssets.ts`        | Yes                          |

## Schema Customization Reference

### Default Values

Set default values directly in the schema:

```typescript
UseMostRecent: {
  type: "boolean",
  default: true,  // ← Extracted to .setDefaultValue(true)
}
```

### Custom Documentation Links

Override the provider's default doc link generation:

```typescript
ExecutableUsers: {
  type: "string",
  description: "...",
  docLink: "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/...",
} as any,  // ← Type assertion needed for custom field
```

**When to use:**
- Manually-managed assets referencing non-CloudFormation APIs
- Linking to specific API documentation sections
- Overriding incorrect auto-generated links

### Custom Array Item Names

Prevent breaking changes by controlling array entry names:

```typescript
Filters: {
  type: "array",
  itemName: "Filter",  // ← Without this, would be "FiltersItem"
  items: {
    type: "object",
    properties: { ... }
  }
} as any,
```

**Critical for backward compatibility!** Default is `{PropertyName}Item`.

### Secret Configuration

Declare which properties are secrets and their kinds:

```typescript
export const schema: CfSchema = {
  properties: {
    credential: {
      type: "string",
      description: "AWS credential",
    },
    apiKey: {
      type: "string",
      description: "API key",
    },
  },
  writeOnlyProperties: [],  // Don't use for secrets
  secretKinds: {
    credential: "AWS Credential",  // ← Explicit mapping
    apiKey: "API Key",
  },
}
```

**Automatic behavior:**
- Properties in `secretKinds` are moved to the secrets section
- `widgetKind` is set to "Secret"
- `widgetOptions` contains the specified secret kind
- Generated code uses `new SecretPropBuilder().setSecretKind("AWS Credential")`

## Common Mistakes

❌ **Using the same function ID twice** → Generate unique IDs for each function
❌ **Changing function IDs** → IDs must be stable, never change them
❌ **Wrong path in function spec** → Path must match actual file location
❌ **Forgetting to register** → Must add to `extraAssets.ts`
❌ **Using writeOnlyProperties for secrets** → Use `secretKinds` instead
❌ **Forgetting `as any` on custom fields** → TypeScript needs type assertion for `docLink`, `itemName`
❌ **Not setting `itemName` on arrays** → Default naming may break user configurations

## Next Steps

1. Decide what resource you want to create
2. Create the directory: `AWS::{Service}::{Resource}`
3. Create `schema.ts` with schema + function specs
4. Implement the functions you declared
5. Register in `extraAssets.ts`
6. Generate and test

That's it! Follow the AWS::EC2::AMI example for a complete working reference.
