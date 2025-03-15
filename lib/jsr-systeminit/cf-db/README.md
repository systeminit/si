# CloudFormation Type Database for System Initiative

This package provides a database of AWS CloudFormation resource types and schemas used by System Initiative. It includes utilities for loading, parsing, and accessing CloudFormation type definitions in a structured and type-safe manner.

## Overview

The CloudFormation database (CF-DB) loads and processes CloudFormation type definitions from JSON schema files. It normalizes and dereferences these schemas to provide a consistent interface for accessing resource properties and metadata.

This library is used by System Initiative to:
- Access CloudFormation resource type definitions
- Understand the structure of AWS resources
- Validate AWS resource configurations
- Generate UI components based on resource properties

## Installation

```bash
# Install via JSR (JavaScript Registry)
npx jsr add @systeminit/cf-db
```

## Key Features

- Load CloudFormation schemas from a directory
- Access resource type definitions by name
- Normalize property types for consistent handling
- Traverse nested property structures
- Type-safe interfaces for CloudFormation resources

## API Reference

### Types

- `CfSchema`: Represents a CloudFormation resource type definition
- `CfProperty`: Represents a property within a CloudFormation resource
- `CfPropertyType`: Union type of supported CloudFormation property types
- Various specialized property types: `CfBooleanProperty`, `CfStringProperty`, etc.

### Functions

- `loadCfDatabase({ path?, services? })`: Loads the CloudFormation database from a directory
  - `path`: Optional path to CloudFormation schema files (defaults to './cloudformation-schema')
  - `services`: Optional array of service name patterns to filter loaded schemas

- `getServiceByName(serviceName)`: Gets a CloudFormation resource type by name
  - Throws `ServiceMissing` if the service isn't found

- `getPropertiesForService(serviceName)`: Gets the properties for a specific service
  - Returns a record of property names to property definitions

- `normalizeProperty(prop)`: Normalizes a CloudFormation property
  - Handles type conversions and special cases

- `allCfProps(root)`: Generator that yields all properties in a schema
  - Traverses nested properties and provides paths

## Usage Example

```typescript
import { loadCfDatabase, getServiceByName } from "@systeminit/cf-db";

// Load the database with default options
await loadCfDatabase({});

// Get a specific service schema
const lambdaSchema = getServiceByName("AWS::Lambda::Function");

// Access properties
const properties = lambdaSchema.properties;
console.log(properties.Handler);  // { type: "string", ... }
```

## Development

See [CLAUDE.md](./CLAUDE.md) for development guidelines, code style, and project commands.

## License

Apache 2.0 (see license information in individual files)