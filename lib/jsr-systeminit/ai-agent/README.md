# System Initiative AI Agent

![License](https://img.shields.io/badge/license-Apache--2.0-blue)
![JSR](https://jsr.io/badges/@systeminit/ai-agent)

AI agent for automating AWS infrastructure provisioning in System Initiative by
interacting with CloudFormation schemas via the AWS Cloud Control API.

## Features

- **Infrastructure Prototyping**: Create AWS infrastructure prototypes from
  natural language descriptions
- **Resource Type Extraction**: Identify appropriate AWS CloudFormation resource
  types for requirements
- **Field Value Extraction**: Parse CloudFormation schemas to populate field
  values
- **Component Editing**: Update existing AWS components with proper
  CloudFormation properties
- **Schema Validation**: Ensure property names match case-sensitive
  CloudFormation requirements

## Installation

```sh
# Import from JSR
import { prototypeInfrastructure } from "jsr:@systeminit/ai-agent";
```

## Usage

### Prototype Infrastructure

Create AWS infrastructure components from natural language descriptions:

```ts
import { prototypeInfrastructure } from "jsr:@systeminit/ai-agent";

// Create infrastructure components from natural language
const result = await prototypeInfrastructure(
  "I need a highly available WordPress site with a load balancer, EC2 instances, and RDS database",
);

console.log(result.ops.create); // Created components
```

### Extract CloudFormation Types

Find appropriate AWS resource types for infrastructure requirements:

```ts
import { extractTypes } from "jsr:@systeminit/ai-agent";

const types = await extractTypes(
  "I need a VPC with public and private subnets",
);

// Returns: [{ cfType: "AWS::EC2::VPC", justification: "..." }, ...]
console.log(types);
```

### Extract Fields from CloudFormation Schemas

Extract field values for AWS resources from natural language:

```ts
import { extractFields } from "jsr:@systeminit/ai-agent";

const fields = await extractFields(
  "AWS::EC2::Instance",
  "Create a t2.micro EC2 instance with Amazon Linux 2",
);

console.log(fields.properties); // Extracted property values
```

### Edit Components

Update existing AWS components with natural language instructions:

```ts
import { editComponent } from "jsr:@systeminit/ai-agent";

const result = await editComponent(
  "myEC2Instance",
  "AWS::EC2::Instance",
  {
    si: { name: "Web Server" },
    domain: { InstanceType: "t2.micro" },
  },
  "Upgrade the instance to t2.large and add a tag for the environment",
);

console.log(result.ops.update); // Updated component properties
```

## API Reference

The library exports the following main functions:

- `prototypeInfrastructure(request: string, maxRetries?: number)`: Build
  infrastructure from descriptions
- `extractTypes(request: string, invalidTypes?: string[])`: Find relevant
  CloudFormation types
- `extractFields(typeName: string, request: string, existingProperties?: Record<string, unknown>)`:
  Extract field values
- `editComponent(componentName: string, kind: string, properties: object, request: string, maxRetries?: number)`:
  Edit components
- `checkPropertyCaseMismatches(kind: string, domain: Record<string, unknown>)`:
  Validate property name casing

## Development

### Build/Lint/Test Commands

- Run all tests: `deno test --allow-all` (note: the full test suite can take
  several minutes)
- Run single test:
  `deno test --allow-all --filter=test_function_name mod_test.ts`
- Run specific tests: `deno test --allow-all --filter="test_pattern"`
- Format code: `deno fmt` or `buck2 run :fix-format`
- Check format: `buck2 run :check-format`
- Lint: `deno lint`
- Publish package: `deno publish` (update version in deno.json first)

## License

Apache-2.0 - See [LICENSE](LICENSE) for details.
