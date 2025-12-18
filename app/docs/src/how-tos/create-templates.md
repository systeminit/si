---
outline: [2,3,4]
---

# How to Create Templates

This guide walks you through creating a System Initiative template from start to finish. Templates allow you to capture working infrastructure patterns and replicate them across workspaces with customization through inputs and transformations.

:::tip

There is a [reference page for templates](../reference/templates.md) that provides comprehensive information on the feature. You may wish to cross-reference this "how-to" with the aforementioned page to bolster your template.

:::

## Prerequisites

1. [System Initiative CLI installed](/tutorials/install-the-cli)
1. A workspace with working infrastructure you want to template
1. The CLI authenticated and using that workspace using `si login` or with a manually created [API token](/explanation/generate-a-workspace-api-token)
1. Basic familiarity with [TypeScript](https://www.typescriptlang.org/)

## What You'll Build

By the end of this "how-to", you'll have created a template that:

1. Searches for specific components in a workspace
1. Accepts input variables to customize the output
1. Renames components based on patterns
1. Transforms component attributes
1. Handles external component references
1. Can be cached and reused across workspaces

## Walkthrough

### Create a Working Example

Before creating a template, you need working infrastructure to use as a baseline. For this guide, we'll assume you have a simple VPC setup with subnets in a workspace.

If you need to create one, follow the [AWS VPC guide](./aws-vpc.md) first.
For this "how-to", we will be using names and examples based on it.

### Generate a New Template

Create a new template file using the CLI:

```shellscript
$ si template generate vpc-pattern
```

This creates a file called `vpc-pattern.ts` with basic structure:

```typescript
import { TemplateContext } from "jsr:@systeminit/si";

export default async function (ctx: TemplateContext) {
  ctx.search(["schema:*"]);
}
```

### Add Component Search

Replace the default `search` payload to target specific components. Use the [search syntax](/reference/search) to find components by schema, name, or tags:

```typescript{4-14}
import { TemplateContext } from "jsr:@systeminit/si";

export default async function (ctx: TemplateContext) {
  ctx.search([
    'schema:"AWS::EC2::EIP"',
    'schema:"AWS::EC2::InternetGateway"',
    'schema:"AWS::EC2::NatGateway"',
    'schema:"AWS::EC2::Route"',
    'schema:"AWS::EC2::RouteTable"',
    'schema:"AWS::EC2::Subnet"',
    'schema:"AWS::EC2::SubnetRouteTableAssociation"',
    'schema:"AWS::EC2::VPC"',
    'schema:"AWS::EC2::VPCGatewayAttachment"'
  ]);
}
```

You can verify what this will select by running the template in dry-run mode:

```shellscript
$ si template run vpc-pattern.ts --key test --dry-run
```

The output will show which components match your search criteria without making any changes to the workspace.

### Add Input Variables

Define `inputs` using [Zod schemas](https://zod.dev/) to make your template configurable:

```typescript{2,5-13}
import { TemplateContext } from "jsr:@systeminit/si";
import { z } from "npm:zod@4";

export default async function (ctx: TemplateContext) {
  // Define input schema
  const inputSchema = z.object({
    environment: z.string().default("dev"),
    vpcCidr: z.string().default("10.0.0.0/16"),
  });
  type Inputs = z.infer<typeof inputSchema>;

  ctx.inputs(inputSchema);

  ctx.search([
    'schema:"AWS::EC2::EIP"',
    'schema:"AWS::EC2::InternetGateway"',
    'schema:"AWS::EC2::NatGateway"',
    'schema:"AWS::EC2::Route"',
    'schema:"AWS::EC2::RouteTable"',
    'schema:"AWS::EC2::Subnet"',
    'schema:"AWS::EC2::SubnetRouteTableAssociation"',
    'schema:"AWS::EC2::VPC"',
    'schema:"AWS::EC2::VPCGatewayAttachment"'
  ]);
}
```

Create an input file `vpc-pattern-inputs.yaml`:

```yaml
environment: prod
vpcCidr: 10.1.0.0/16
```

You can dry-run the template again to verify that your inputs are being parsed correctly:

```shellscript
$ si template run vpc-pattern.ts --key test --input vpc-pattern-inputs.yaml --dry-run
```

The dry-run output will show the matched components and confirm your input values are valid.

### Add a Name Pattern

Use `namePattern` to rename components based on input variables:

```typescript{25-28}
import { TemplateContext } from "jsr:@systeminit/si";
import { z } from "npm:zod@4";

export default async function (ctx: TemplateContext) {
  const inputSchema = z.object({
    environment: z.string().default("dev"),
    vpcCidr: z.string().default("10.0.0.0/16"),
  });
  type Inputs = z.infer<typeof inputSchema>;

  ctx.inputs(inputSchema);

  ctx.search([
    'schema:"AWS::EC2::EIP"',
    'schema:"AWS::EC2::InternetGateway"',
    'schema:"AWS::EC2::NatGateway"',
    'schema:"AWS::EC2::Route"',
    'schema:"AWS::EC2::RouteTable"',
    'schema:"AWS::EC2::Subnet"',
    'schema:"AWS::EC2::SubnetRouteTableAssociation"',
    'schema:"AWS::EC2::VPC"',
    'schema:"AWS::EC2::VPCGatewayAttachment"'
  ]);

  // Rename components from "demo-*" to "{environment}-*"
  ctx.namePattern([
    { pattern: /demo-(.+)/, replacement: "<%= inputs.environment %>-$1" },
  ]);
}
```

The replacement pattern uses [EJS syntax](https://ejs.co/) to reference input variables. We can see if this works by performing a dry-run:

```shellscript
$ si template run vpc-pattern.ts --key test --input vpc-pattern-inputs.yaml --dry-run
```

The output will show how component names will be transformed (e.g., "demo-vpc" becomes "prod-vpc").

### Add a Transform

The `transform` function is where you modify component attributes, create new components, or filter the working set:

```typescript{29-70}
import { TemplateContext } from "jsr:@systeminit/si";
import { z } from "npm:zod@4";

export default async function (ctx: TemplateContext) {
  const inputSchema = z.object({
    environment: z.string().default("dev"),
    vpcCidr: z.string().default("10.0.0.0/16"),
  });
  type Inputs = z.infer<typeof inputSchema>;

  ctx.inputs(inputSchema);

  ctx.search([
    'schema:"AWS::EC2::EIP"',
    'schema:"AWS::EC2::InternetGateway"',
    'schema:"AWS::EC2::NatGateway"',
    'schema:"AWS::EC2::Route"',
    'schema:"AWS::EC2::RouteTable"',
    'schema:"AWS::EC2::Subnet"',
    'schema:"AWS::EC2::SubnetRouteTableAssociation"',
    'schema:"AWS::EC2::VPC"',
    'schema:"AWS::EC2::VPCGatewayAttachment"'
  ]);

  ctx.namePattern([
    { pattern: /demo-(.+)/, replacement: "<%= inputs.environment %>-$1" },
  ]);

  ctx.transform(async (workingSet, inputs) => {
    const typedInputs = inputs as Inputs;

    // Update VPC CIDR block from inputs
    const vpc = workingSet.find((c) => c.schemaName === "AWS::EC2::VPC");
    if (vpc) {
      await ctx.ensureAttribute(vpc, "/domain/CidrBlock", typedInputs.vpcCidr);
    }

    // Update subnet CIDR blocks to match new VPC range
    // Extract the base IP from vpcCidr (e.g., "10.1" from "10.1.0.0/16")
    const vpcBase = typedInputs.vpcCidr.split('.').slice(0, 2).join('.');

    const subnets = workingSet.filter((c) => c.schemaName === "AWS::EC2::Subnet");
    const publicSubnets = subnets.filter(s => s.name.includes("public"));
    const privateSubnets = subnets.filter(s => s.name.includes("private"));

    // Assign /20 blocks for public subnets (0, 16, 32...)
    publicSubnets.forEach(async (subnet, index) => {
      const subnetCidr = `${vpcBase}.${index * 16}.0/20`;
      await ctx.ensureAttribute(subnet, "/domain/CidrBlock", subnetCidr);
    });

    // Assign /20 blocks for private subnets (128, 144, 160...)
    privateSubnets.forEach(async (subnet, index) => {
      const subnetCidr = `${vpcBase}.${128 + (index * 16)}.0/20`;
      await ctx.ensureAttribute(subnet, "/domain/CidrBlock", subnetCidr);
    });

    // Add environment tags to all components
    for (const component of workingSet) {
      await ctx.ensureArrayAttribute(
        component,
        "/domain/Tags",
        (e) => e.subpath === "Key" && e.value === "Environment",
        { Key: "Environment", Value: typedInputs.environment },
        { skipIfMissing: true }
      );
    }

    return workingSet;
  });
}
```

Perform a dry-run test to preview attribute changes:

```shellscript
$ si template run vpc-pattern.ts --key test --input vpc-pattern-inputs.yaml --dry-run
```

You'll see the VPC CIDR update and environment tags being applied to all components.

### Add External Component References

For templates to work access different workspaces, you need to handle component references not included in your template (e.g. `AWS Credential` and `AWS Region` in the case of AWS VPC) as inputs. By default, if a subscription is not in the working set, the templating system will attempt to use it unmodified. As a result, the template won't work in another workspace _unless_ we handle this scenario, which can be done with `SubscriptionInput`:

```typescript{71-86}
import { SubscriptionInput, TemplateContext } from "jsr:@systeminit/si";
import { z } from "npm:zod@4";

export default async function (ctx: TemplateContext) {
  const inputSchema = z.object({
    environment: z.string().default("dev"),
    vpcCidr: z.string().default("10.0.0.0/16"),
    credential: SubscriptionInput,
    region: SubscriptionInput,
  });
  type Inputs = z.infer<typeof inputSchema>;

  ctx.inputs(inputSchema);

  ctx.search([
    'schema:"AWS::EC2::EIP"',
    'schema:"AWS::EC2::InternetGateway"',
    'schema:"AWS::EC2::NatGateway"',
    'schema:"AWS::EC2::Route"',
    'schema:"AWS::EC2::RouteTable"',
    'schema:"AWS::EC2::Subnet"',
    'schema:"AWS::EC2::SubnetRouteTableAssociation"',
    'schema:"AWS::EC2::VPC"',
    'schema:"AWS::EC2::VPCGatewayAttachment"'
  ]);

  ctx.namePattern([
    { pattern: /demo-(.+)/, replacement: "<%= inputs.environment %>-$1" },
  ]);

  ctx.transform(async (workingSet, inputs) => {
    const typedInputs = inputs as Inputs;

    // Update VPC CIDR block from inputs
    const vpc = workingSet.find((c) => c.schemaName === "AWS::EC2::VPC");
    if (vpc) {
      await ctx.ensureAttribute(vpc, "/domain/CidrBlock", typedInputs.vpcCidr);
    }

    // Update subnet CIDR blocks to match new VPC range
    // Extract the base IP from vpcCidr (e.g., "10.1" from "10.1.0.0/16")
    const vpcBase = typedInputs.vpcCidr.split('.').slice(0, 2).join('.');

    const subnets = workingSet.filter((c) => c.schemaName === "AWS::EC2::Subnet");
    const publicSubnets = subnets.filter(s => s.name.includes("public"));
    const privateSubnets = subnets.filter(s => s.name.includes("private"));

    // Assign /20 blocks for public subnets (0, 16, 32...)
    publicSubnets.forEach(async (subnet, index) => {
      const subnetCidr = `${vpcBase}.${index * 16}.0/20`;
      await ctx.ensureAttribute(subnet, "/domain/CidrBlock", subnetCidr);
    });

    // Assign /20 blocks for private subnets (128, 144, 160...)
    privateSubnets.forEach(async (subnet, index) => {
      const subnetCidr = `${vpcBase}.${128 + (index * 16)}.0/20`;
      await ctx.ensureAttribute(subnet, "/domain/CidrBlock", subnetCidr);
    });

    // Add environment tags to all components
    for (const component of workingSet) {
      await ctx.ensureArrayAttribute(
        component,
        "/domain/Tags",
        (e) => e.subpath === "Key" && e.value === "Environment",
        { Key: "Environment", Value: typedInputs.environment },
        { skipIfMissing: true }
      );
    }

    // Set credential and region subscriptions for all components
    for (const component of workingSet) {
      await ctx.ensureAttribute(
        component,
        "/secrets/AWS Credential",
        typedInputs.credential,
        { skipIfMissing: true }
      );

      await ctx.ensureAttribute(
        component,
        "/domain/extra/Region",
        typedInputs.region,
        { skipIfMissing: true }
      );
    }

    return workingSet;
  });
}
```

Update your input file to specify how to find these components:

```yaml
environment: prod
vpcCidr: 10.1.0.0/16
credential:
  kind: "search"
  query: 'schema:"AWS Credential"'
  path: "/secrets/AWS Credential"
region:
  kind: "search"
  query: 'schema:"Region"'
  path: "/domain/region"
```

Perform a dry-run test with the updated inputs to verify external component references resolve correctly:

```shellscript
$ si template run vpc-pattern.ts --key test --input vpc-pattern-inputs.yaml --dry-run
```

The output will confirm that AWS Credential and Region subscriptions are being set properly on all components.

### Run it and Make it Reusable with a Baseline Cache

We are now ready to run without the `--dry-run` flag. We not only do that, but make it reusable across workspaces in one command. You can do this by creating a baseline cache at the same time:

```shellscript
$ si template run vpc-pattern.ts \
  --input vpc-pattern-inputs.yaml \
  --key cache \
  --cache-baseline vpc-pattern-baseline.yaml \
  --cache-baseline-only
```

The final output should look like this:

```shellscript
✨ info    si              Loading Template: "file:///home/toddhoward/templates/vpc-pattern.ts"
✨ info    si              Loading input data from: "vpc-pattern-inputs.yaml"
✨ info    si              Building baseline with search strings: [ 'schema:"AWS::EC2::EIP"',
                             'schema:"AWS::EC2::InternetGateway"',
                             'schema:"AWS::EC2::NatGateway"',
                             'schema:"AWS::EC2::Route"',
                             'schema:"AWS::EC2::RouteTable"',
                             'schema:"AWS::EC2::Subnet"',
                             'schema:"AWS::EC2::SubnetRouteTableAssociation"',
                             'schema:"AWS::EC2::VPC"',
                             'schema:"AWS::EC2::VPCGatewayAttachment"' ]
✨ info    si              Found 29 unique components from search
✨ info    si              Loaded baseline component "AWS::EC2::EIP" "demo-eip-natgw-1a" (1/29)
✨ info    si              Loaded baseline component "AWS::EC2::EIP" "demo-eip-natgw-1b" (2/29)
✨ info    si              Loaded baseline component "AWS::EC2::EIP" "demo-eip-natgw-1c" (3/29)
✨ info    si              Loaded baseline component "AWS::EC2::InternetGateway" "demo-igw" (4/29)
✨ info    si              Loaded baseline component "AWS::EC2::NatGateway" "demo-natgw-1a" (5/29)
✨ info    si              Loaded baseline component "AWS::EC2::NatGateway" "demo-natgw-1b" (6/29)
✨ info    si              Loaded baseline component "AWS::EC2::NatGateway" "demo-natgw-1c" (7/29)
✨ info    si              Loaded baseline component "AWS::EC2::Route" "demo-private-route-natgw-1b" (8/29)
✨ info    si              Loaded baseline component "AWS::EC2::Route" "demo-private-route-natgw-1c" (9/29)
✨ info    si              Loaded baseline component "AWS::EC2::Route" "demo-public-route-igw" (10/29)
✨ info    si              Loaded baseline component "AWS::EC2::Route" "demo-private-route-natgw-1a" (11/29)
✨ info    si              Loaded baseline component "AWS::EC2::RouteTable" "demo-public-rtb" (12/29)
✨ info    si              Loaded baseline component "AWS::EC2::RouteTable" "demo-private-rtb-1a" (13/29)
✨ info    si              Loaded baseline component "AWS::EC2::RouteTable" "demo-private-rtb-1b" (14/29)
✨ info    si              Loaded baseline component "AWS::EC2::RouteTable" "demo-private-rtb-1c" (15/29)
✨ info    si              Loaded baseline component "AWS::EC2::Subnet" "demo-private-subnet-1b" (16/29)
✨ info    si              Loaded baseline component "AWS::EC2::Subnet" "demo-public-subnet-1c" (17/29)
✨ info    si              Loaded baseline component "AWS::EC2::Subnet" "demo-private-subnet-1c" (18/29)
✨ info    si              Loaded baseline component "AWS::EC2::Subnet" "demo-public-subnet-1a" (19/29)
✨ info    si              Loaded baseline component "AWS::EC2::Subnet" "demo-private-subnet-1a" (20/29)
✨ info    si              Loaded baseline component "AWS::EC2::Subnet" "demo-public-subnet-1b" (21/29)
✨ info    si              Loaded baseline component "AWS::EC2::SubnetRouteTableAssociation" "demo-private-subnet-1c-rtb-assoc" (22/29)
✨ info    si              Loaded baseline component "AWS::EC2::SubnetRouteTableAssociation" "demo-public-subnet-1a-rtb-assoc" (23/29)
✨ info    si              Loaded baseline component "AWS::EC2::SubnetRouteTableAssociation" "demo-private-subnet-1b-rtb-assoc" (24/29)
✨ info    si              Loaded baseline component "AWS::EC2::SubnetRouteTableAssociation" "demo-public-subnet-1b-rtb-assoc" (25/29)
✨ info    si              Loaded baseline component "AWS::EC2::SubnetRouteTableAssociation" "demo-private-subnet-1a-rtb-assoc" (26/29)
✨ info    si              Loaded baseline component "AWS::EC2::SubnetRouteTableAssociation" "demo-public-subnet-1c-rtb-assoc" (27/29)
✨ info    si              Loaded baseline component "AWS::EC2::VPC" "demo-vpc" (28/29)
✨ info    si              Loaded baseline component "AWS::EC2::VPCGatewayAttachment" "demo-igw-attachment" (29/29)
✨ info    si              Built baseline with 29 components from search
✨ info    si              Caching baseline to "vpc-pattern-baseline.yaml"
✨ info    si              Cached 29 components and 9 schemas to "vpc-pattern-baseline.yaml"
✨ info    si              Baseline cache written successfully. Exiting.
```

This saves all component data to `vpc-pattern-baseline.yaml`. You can commit this file alongside your template in version control.

### Run Your Template In Another Workspace

:::tip

You can switch workspaces with the `si` CLI directly:

```shellscript
$ si workspace switch
```

:::

Now, you can run the template in any workspace using the cached baseline. Ensure that you have the necessary components in place first (e.g. `AWS Credential` and `AWS Region` if you are creating an AWS VPC).


```shellscript
$ si template run vpc-pattern.ts \
  --key new-vpc \
  --baseline vpc-pattern-baseline.yaml \
  --input vpc-pattern-inputs.yaml
```

This creates infrastructure matching your baseline without needing the original components. The output should look like this:

```shellscript
✨ info    si              Loading Template: "file:///home/toddhoward/templates/vpc-pattern.ts"
✨ info    si              Loading input data from: "vpc-pattern-inputs.yaml"
✨ info    si              Loading baseline data from "vpc-pattern-baseline.yaml"
✨ info    si              Initializing working set: 29 components
✨ info    si              Applying pattern 1/1: "demo-(.+)" -> "prod-$1"
✨ info    si              Executing transformation function
✨ info    si              Ensuring "AWS::EC2::VPC" "prod-vpc" "/domain/CidrBlock" has "10.1.0.0/16"
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-private-subnet-1b" "/domain/CidrBlock" has "10.1.128.0/20"
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-private-subnet-1c" "/domain/CidrBlock" has "10.1.144.0/20"
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-public-subnet-1c" "/domain/CidrBlock" has "10.1.0.0/20"
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-public-subnet-1a" "/domain/CidrBlock" has "10.1.16.0/20"
✨ info    si              Ensuring "AWS::EC2::EIP" "prod-eip-natgw-1a" "/domain/Tags" array element has { Key: "Environment", Value: "prod" }
✨ info    si              Ensuring "AWS::EC2::EIP" "prod-eip-natgw-1b" "/domain/Tags" array element has { Key: "Environment", Value: "prod" }
✨ info    si              Ensuring "AWS::EC2::EIP" "prod-eip-natgw-1c" "/domain/Tags" array element has { Key: "Environment", Value: "prod" }
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-private-subnet-1a" "/domain/CidrBlock" has "10.1.160.0/20"
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-public-subnet-1b" "/domain/CidrBlock" has "10.1.32.0/20"
✨ info    si              Ensuring "AWS::EC2::InternetGateway" "prod-igw" "/domain/Tags" array element has { Key: "Environment", Value: "prod" }
✨ info    si              Ensuring "AWS::EC2::NatGateway" "prod-natgw-1a" "/domain/Tags" array element has { Key: "Environment", Value: "prod" }
✨ info    si              Ensuring "AWS::EC2::NatGateway" "prod-natgw-1b" "/domain/Tags" array element has { Key: "Environment", Value: "prod" }
✨ info    si              Ensuring "AWS::EC2::NatGateway" "prod-natgw-1c" "/domain/Tags" array element has { Key: "Environment", Value: "prod" }
✨ info    si              Ensuring "AWS::EC2::RouteTable" "prod-public-rtb" "/domain/Tags" array element has { Key: "Environment", Value: "prod" }
✨ info    si              Ensuring "AWS::EC2::RouteTable" "prod-private-rtb-1a" "/domain/Tags" array element has { Key: "Environment", Value: "prod" }
✨ info    si              Ensuring "AWS::EC2::RouteTable" "prod-private-rtb-1b" "/domain/Tags" array element has { Key: "Environment", Value: "prod" }
✨ info    si              Ensuring "AWS::EC2::RouteTable" "prod-private-rtb-1c" "/domain/Tags" array element has { Key: "Environment", Value: "prod" }
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-private-subnet-1b" "/domain/Tags" array element has { Key: "Environment", Value: "prod" }
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-public-subnet-1c" "/domain/Tags" array element has { Key: "Environment", Value: "prod" }
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-private-subnet-1c" "/domain/Tags" array element has { Key: "Environment", Value: "prod" }
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-public-subnet-1a" "/domain/Tags" array element has { Key: "Environment", Value: "prod" }
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-private-subnet-1a" "/domain/Tags" array element has { Key: "Environment", Value: "prod" }
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-public-subnet-1b" "/domain/Tags" array element has { Key: "Environment", Value: "prod" }
✨ info    si              Ensuring "AWS::EC2::VPC" "prod-vpc" "/domain/Tags" array element has { Key: "Environment", Value: "prod" }
✨ info    si              Ensuring "AWS::EC2::EIP" "prod-eip-natgw-1a" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::EIP" "prod-eip-natgw-1a" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::EIP" "prod-eip-natgw-1b" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::EIP" "prod-eip-natgw-1b" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::EIP" "prod-eip-natgw-1c" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::EIP" "prod-eip-natgw-1c" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::InternetGateway" "prod-igw" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::InternetGateway" "prod-igw" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::NatGateway" "prod-natgw-1a" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::NatGateway" "prod-natgw-1a" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::NatGateway" "prod-natgw-1b" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::NatGateway" "prod-natgw-1b" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::NatGateway" "prod-natgw-1c" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::NatGateway" "prod-natgw-1c" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::Route" "prod-private-route-natgw-1b" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::Route" "prod-private-route-natgw-1b" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::Route" "prod-private-route-natgw-1c" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::Route" "prod-private-route-natgw-1c" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::Route" "prod-public-route-igw" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::Route" "prod-public-route-igw" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::Route" "prod-private-route-natgw-1a" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::Route" "prod-private-route-natgw-1a" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::RouteTable" "prod-public-rtb" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::RouteTable" "prod-public-rtb" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::RouteTable" "prod-private-rtb-1a" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::RouteTable" "prod-private-rtb-1a" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::RouteTable" "prod-private-rtb-1b" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::RouteTable" "prod-private-rtb-1b" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::RouteTable" "prod-private-rtb-1c" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::RouteTable" "prod-private-rtb-1c" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-private-subnet-1b" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-private-subnet-1b" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-public-subnet-1c" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-public-subnet-1c" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-private-subnet-1c" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-private-subnet-1c" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-public-subnet-1a" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-public-subnet-1a" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-private-subnet-1a" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-private-subnet-1a" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-public-subnet-1b" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::Subnet" "prod-public-subnet-1b" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::SubnetRouteTableAssociation" "prod-private-subnet-1c-rtb-assoc" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::SubnetRouteTableAssociation" "prod-private-subnet-1c-rtb-assoc" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::SubnetRouteTableAssociation" "prod-public-subnet-1a-rtb-assoc" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::SubnetRouteTableAssociation" "prod-public-subnet-1a-rtb-assoc" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::SubnetRouteTableAssociation" "prod-private-subnet-1b-rtb-assoc" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::SubnetRouteTableAssociation" "prod-private-subnet-1b-rtb-assoc" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::SubnetRouteTableAssociation" "prod-public-subnet-1b-rtb-assoc" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::SubnetRouteTableAssociation" "prod-public-subnet-1b-rtb-assoc" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::SubnetRouteTableAssociation" "prod-private-subnet-1a-rtb-assoc" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::SubnetRouteTableAssociation" "prod-private-subnet-1a-rtb-assoc" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::SubnetRouteTableAssociation" "prod-public-subnet-1c-rtb-assoc" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::SubnetRouteTableAssociation" "prod-public-subnet-1c-rtb-assoc" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::VPC" "prod-vpc" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::VPC" "prod-vpc" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Ensuring "AWS::EC2::VPCGatewayAttachment" "prod-igw-attachment" "/secrets/AWS Credential" has { "$source": { component: "01KCWCXS1BRYJ1QZC8EXRKVYEX", path: "/secrets/AWS Credential" } }
✨ info    si              Ensuring "AWS::EC2::VPCGatewayAttachment" "prod-igw-attachment" "/domain/extra/Region" has { "$source": { component: "01KCWCXAS7PQPHWDVESP25TVFP", path: "/domain/region" } }
✨ info    si              Getting or creating change set: "vpc-pattern-new-vpc"
✨ info    si              Found 0 existing components
✨ info    si              Computing delta
✨ info    si              Pending changes: 29 creates, 0 updates, 0 deletes
✨ info    si              Creating "AWS::EC2::EIP" "prod-eip-natgw-1a" (1/29)
✨ info    si              Creating "AWS::EC2::EIP" "prod-eip-natgw-1b" (2/29)
✨ info    si              Creating "AWS::EC2::EIP" "prod-eip-natgw-1c" (3/29)
✨ info    si              Creating "AWS::EC2::InternetGateway" "prod-igw" (4/29)
✨ info    si              Creating "AWS::EC2::VPC" "prod-vpc" (5/29)
✨ info    si              Creating "AWS::EC2::RouteTable" "prod-public-rtb" (6/29)
✨ info    si              Creating "AWS::EC2::RouteTable" "prod-private-rtb-1a" (7/29)
✨ info    si              Creating "AWS::EC2::RouteTable" "prod-private-rtb-1b" (8/29)
✨ info    si              Creating "AWS::EC2::RouteTable" "prod-private-rtb-1c" (9/29)
✨ info    si              Creating "AWS::EC2::Subnet" "prod-private-subnet-1b" (10/29)
✨ info    si              Creating "AWS::EC2::Subnet" "prod-public-subnet-1c" (11/29)
✨ info    si              Creating "AWS::EC2::Subnet" "prod-private-subnet-1c" (12/29)
✨ info    si              Creating "AWS::EC2::Subnet" "prod-public-subnet-1a" (13/29)
✨ info    si              Creating "AWS::EC2::Subnet" "prod-private-subnet-1a" (14/29)
✨ info    si              Creating "AWS::EC2::Subnet" "prod-public-subnet-1b" (15/29)
✨ info    si              Creating "AWS::EC2::VPCGatewayAttachment" "prod-igw-attachment" (16/29)
✨ info    si              Creating "AWS::EC2::Route" "prod-public-route-igw" (17/29)
✨ info    si              Creating "AWS::EC2::SubnetRouteTableAssociation" "prod-private-subnet-1b-rtb-assoc" (18/29)
✨ info    si              Creating "AWS::EC2::NatGateway" "prod-natgw-1c" (19/29)
✨ info    si              Creating "AWS::EC2::SubnetRouteTableAssociation" "prod-public-subnet-1c-rtb-assoc" (20/29)
✨ info    si              Creating "AWS::EC2::SubnetRouteTableAssociation" "prod-private-subnet-1c-rtb-assoc" (21/29)
✨ info    si              Creating "AWS::EC2::NatGateway" "prod-natgw-1a" (22/29)
✨ info    si              Creating "AWS::EC2::SubnetRouteTableAssociation" "prod-public-subnet-1a-rtb-assoc" (23/29)
✨ info    si              Creating "AWS::EC2::SubnetRouteTableAssociation" "prod-private-subnet-1a-rtb-assoc" (24/29)
✨ info    si              Creating "AWS::EC2::NatGateway" "prod-natgw-1b" (25/29)
✨ info    si              Creating "AWS::EC2::SubnetRouteTableAssociation" "prod-public-subnet-1b-rtb-assoc" (26/29)
✨ info    si              Creating "AWS::EC2::Route" "prod-private-route-natgw-1c" (27/29)
✨ info    si              Creating "AWS::EC2::Route" "prod-private-route-natgw-1a" (28/29)
✨ info    si              Creating "AWS::EC2::Route" "prod-private-route-natgw-1b" (29/29)
✨ info    si              Execution complete: 29 succeeded, 0 failed
```

Congratulations! Not only did you run a template in another workspace, but you now have a reusable template that you can use in other workspaces!

## Where Can I Learn More?

The [reference page](../reference/templates.md) contains comprehensive documentation on the templates. Within that document, you can find common patterns, best patterns, tips, tricks and more.
