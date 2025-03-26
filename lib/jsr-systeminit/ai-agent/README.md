# System Initiative AI Agent

![License](https://img.shields.io/badge/license-Apache--2.0-blue)
![JSR](https://jsr.io/badges/@systeminit/ai-agent)

AI agent for automating AWS infrastructure provisioning in System Initiative by
interacting with CloudFormation schemas via the AWS Cloud Control API.

## Features

- **Field Value Extraction**: Parse CloudFormation schemas to populate field
  values
- **Component Editing**: Update existing AWS components with proper
  CloudFormation properties
- **Schema Validation**: Ensure property names match case-sensitive
  CloudFormation requirements
- **Schema-Driven Validation**: Include schema definitions for better type accuracy
- **Nested Structure Support**: Handle deeply nested properties, arrays, and map types
- **Two-Phase Editing**: Propose changes first, then apply them

## Installation

```sh
# Import from JSR
import { editComponent } from "jsr:@systeminit/ai-agent";
```

## Usage

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

### Propose Component Edits

Generate component edit suggestions without applying changes:

```ts
import { proposeEdits } from "jsr:@systeminit/ai-agent";

const suggestions = await proposeEdits(
  "AWS::EC2::Instance",
  {
    si: { name: "Web Server" },
    domain: { InstanceType: "t2.micro" },
  },
  "Upgrade the instance to t2.large and add a tag for the environment",
);

console.log(suggestions.properties); // Suggested property changes
```

### Edit Components

Update existing AWS components with natural language instructions:

```ts
import { editComponent, proposeEdits } from "jsr:@systeminit/ai-agent";

// Two-step approach with review
const suggestions = await proposeEdits(
  "AWS::EC2::Instance",
  {
    si: { name: "Web Server" },
    domain: { InstanceType: "t2.micro" },
  },
  "Upgrade the instance to t2.large and add a tag for the environment",
);

// Review suggestions here...
console.log(suggestions.properties);

// Then apply changes
const result = await editComponent(
  "myEC2Instance",
  "AWS::EC2::Instance",
  {
    si: { name: "Web Server" },
    domain: { InstanceType: "t2.micro" },
  },
  "Upgrade the instance to t2.large and add a tag for the environment",
  suggestions, // Pass the suggestions to avoid regenerating them
);

console.log(result.ops.update); // Updated component properties

// One-step approach (immediate application)
const directResult = await editComponent(
  "myEC2Instance",
  "AWS::EC2::Instance",
  {
    si: { name: "Web Server" },
    domain: { InstanceType: "t2.micro" },
  },
  "Upgrade the instance to t2.large and add a tag for the environment",
);

console.log(directResult.ops.update); // Updated component properties
```

### Advanced Example: Handling Deeply Nested Structures

The library now includes enhanced schema traversal for complex nested structures, array indices, and map types with arbitrary keys:

```ts
import { editComponent } from "jsr:@systeminit/ai-agent";

// Edit an ECS Task Definition with nested container definitions and log configuration
const result = await editComponent(
  "appTaskDefinition",
  "AWS::ECS::TaskDefinition",
  {
    si: { 
      name: "Application Task Definition",
      type: "aws-ecs-taskdefinition"
    },
    domain: {
      Family: "app-task",
      ContainerDefinitions: [
        {
          Name: "app-container",
          Image: "nginx:latest",
          Essential: true,
        }
      ]
    }
  },
  "Add CloudWatch logging to the first container definition with log group '/ecs/app-logs' and region 'us-west-2'"
);

// The result will include properly structured deeply nested properties:
// - ContainerDefinitions[0].LogConfiguration.LogDriver = "awslogs"
// - ContainerDefinitions[0].LogConfiguration.Options = { "awslogs-group": "/ecs/app-logs", ... }
```

## API Reference

The library exports the following main functions:

- `extractFields(typeName: string, request: string, existingProperties?: Record<string, unknown>, maxRetries?: number)`:
  Extract field values
- `proposeEdits(kind: string, properties: object, request: string)`:
  Propose component edits without applying them
- `editComponent(componentName: string, kind: string, properties: object, request: string, extractResponse?: ExtractFieldsResponse, maxRetries?: number)`:
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
