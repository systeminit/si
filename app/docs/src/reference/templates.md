---
outline: [2,3,4]
---

# Templates

## Introduction

Templates allow you to use the components in a workspace as a pattern for creating or updating new components. A template consists of the baseline components you want to use as a pattern, optional input variables, and the logic for how to transform the baseline components into new components. Templates are written as TypeScript functions in a declarative style and executed with the `si` command line tool. They are typically stored in git.

:::tip
Templates are written in a declarative internal TypeScript DSL. You build up the behavior you want, and then the template engine executes it on your behalf.
:::

When a template is run, it will:

- Build the set of "baseline" components, either by searching a workspace or reading a cached YAML file
- Load any input variables you define
- Apply transformations to the baseline, creating a "working set" of components that should be created or updated
- Update any subscriptions that are inclusive of the working set to point to the new components
- Create a [Change Set](./change-sets.md)
- Search for existing components that were previously created by this template, creating the "existing set"
- Compare the "working set" to the "existing set", determining what components need to be created, updated, or deleted
- Perform the create, update, or delete operations in the Change Set

Each invocation of a template uses a supplied "invocation key" to both name the change set and determine which components should be included in the existing set.

:::info
Templates are idempotent (they only make the changes they need to) and convergent (they move from an incorrect state to a correct state) when run repeatedly in a workspace with the same invocation key. For example, if you applied a "Standard VPC" template to a workspace with the invocation key "default-vpc", ran it, and then updated the template to remove NAT Gateways, the NAT Gateways created in the initial run would be deleted.

This makes templates easy to iterate on, and useful as a tool for ongoing evolution of the components they create (even after they are applied).
:::

The general workflow in developing templates is:

1. Create or Discover a working implementation in a workspace
2. Write the template that applies transformations to your working example
3. Iterate on your template by running it repeatedly with the same invocation key
4. Create a baseline cache of the working example
5. Run the template in another workspace, using the baseline cache

## Generating new templates

<DocTabs tabs="CLI">
<TabPanel value="CLI">

To generate a new template with the CLI:

```shellscript [Create a Template]
$ si template generate rebuild
✨ info    si              Generating template structure
✨ info    si              Template generated successfully

Template generated: /home/adam/src/template-test/rebuild.ts

Next steps:
1. Edit /home/adam/src/template-test/rebuild.ts to customize your template
2. Run your template with: si template run /home/adam/src/template-test/rebuild.ts
```

</TabPanel>
</DocTabs>

The resulting template file will contain:

```typescript [rebuild.ts]
import { TemplateContext } from "jsr:@systeminit/si";

export default async function (ctx: TemplateContext) {
  ctx.search(["schema:*"]);
}
```

With no changes, this template will make a replica of the workspace it is run against.

:::info
This is because templates use the baseline as the source of components to create. Since `schema:*` matches all components, this template will create a copy of every component in the workspace in a new change set. This assumes the baseline components were not created by a previous execution of this template with the same invocation key; if they were, it would do nothing (because the components are already identical and templates are idempotent).
:::

## Running templates

<DocTabs tabs="CLI">
<TabPanel value="CLI">

To run a template with the CLI:

```shellscript [Create a Template]
$ si template run rebuild.ts --key recreate
✨ info    si              Loading Template: "file:///home/adam/src/template-test/rebuild.ts"
✨ info    si              Building baseline with search strings: [ 'schema:*' ]
✨ info    si              Found 17 unique components from search
✨ info    si              Loaded baseline component "AWS::EC2::SubnetRouteTableAssociation" "demo-subnet-1-route-association" (1/17)
✨ info    si              Loaded baseline component "AWS::EC2::KeyPair" "demo-keypair" (2/17)
✨ info    si              Loaded baseline component "AWS::EC2::Subnet" "demo-subnet-1" (3/17)
✨ info    si              Loaded baseline component "AWS::EC2::VPC" "demo-vpc" (4/17)
✨ info    si              Loaded baseline component "AWS::EC2::Subnet" "demo-subnet-2" (5/17)
✨ info    si              Loaded baseline component "AWS::EC2::Route" "demo-internet-route" (6/17)
✨ info    si              Loaded baseline component "AWS::EC2::RouteTable" "demo-route-table" (7/17)
✨ info    si              Loaded baseline component "AWS::EC2::InternetGateway" "demo-igw" (8/17)
✨ info    si              Loaded baseline component "Region" "demo virginia" (9/17)
✨ info    si              Loaded baseline component "AWS::ElasticLoadBalancingV2::TargetGroup" "demo-target-group-ip" (10/17)
✨ info    si              Loaded baseline component "AWS::EC2::SecurityGroup" "demo-web-sg" (11/17)
✨ info    si              Loaded baseline component "AWS::EC2::Instance" "demo-web-server-1" (12/17)
✨ info    si              Loaded baseline component "AWS::EC2::Instance" "demo-web-server-2" (13/17)
✨ info    si              Loaded baseline component "AWS::ElasticLoadBalancingV2::LoadBalancer" "demo-load-balancer" (14/17)
✨ info    si              Loaded baseline component "AWS::ElasticLoadBalancingV2::Listener" "demo-alb-listener" (15/17)
✨ info    si              Loaded baseline component "AWS::EC2::VPCGatewayAttachment" "demo-igw-attachment" (16/17)
✨ info    si              Loaded baseline component "AWS::EC2::SubnetRouteTableAssociation" "demo-subnet-2-route-association" (17/17)
✨ info    si              Built baseline with 17 components from search
✨ info    si              Initializing working set: 17 components
✨ info    si              Getting or creating change set: "rebuild-recreate"
✨ info    si              Found 0 existing components
✨ info    si              Computing delta
✨ info    si              Pending changes: 17 creates, 0 updates, 0 deletes
✨ info    si              Creating "Region" "demo virginia" (1/17)
✨ info    si              Creating "AWS::EC2::KeyPair" "demo-keypair" (2/17)
✨ info    si              Creating "AWS::EC2::VPC" "demo-vpc" (3/17)
✨ info    si              Creating "AWS::EC2::InternetGateway" "demo-igw" (4/17)
✨ info    si              Creating "AWS::EC2::Subnet" "demo-subnet-1" (5/17)
✨ info    si              Creating "AWS::EC2::Subnet" "demo-subnet-2" (6/17)
✨ info    si              Creating "AWS::EC2::RouteTable" "demo-route-table" (7/17)
✨ info    si              Creating "AWS::EC2::SecurityGroup" "demo-web-sg" (8/17)
✨ info    si              Creating "AWS::EC2::VPCGatewayAttachment" "demo-igw-attachment" (9/17)
✨ info    si              Creating "AWS::EC2::SubnetRouteTableAssociation" "demo-subnet-1-route-association" (10/17)
✨ info    si              Creating "AWS::EC2::Route" "demo-internet-route" (11/17)
✨ info    si              Creating "AWS::EC2::SubnetRouteTableAssociation" "demo-subnet-2-route-association" (12/17)
✨ info    si              Creating "AWS::EC2::Instance" "demo-web-server-1" (13/17)
✨ info    si              Creating "AWS::EC2::Instance" "demo-web-server-2" (14/17)
✨ info    si              Creating "AWS::ElasticLoadBalancingV2::LoadBalancer" "demo-load-balancer" (15/17)
✨ info    si              Creating "AWS::ElasticLoadBalancingV2::TargetGroup" "demo-target-group-ip" (16/17)
✨ info    si              Creating "AWS::ElasticLoadBalancingV2::Listener" "demo-alb-listener" (17/17)
✨ info    si              Execution complete: 17 succeeded, 0 failed
```
</TabPanel>
</DocTabs>

Running this template will create a 'rebuild-recreate' change set with 17 new components, each an exact copy of the existing components, and any subscriptions between them will be updated to point to the new components.

:::info
If a component has a subscription to another component that also has an entry in the working set, its subscription will be updated to point to its peer in the working set. If it has a subscription to a component that is not in the working set, it will remain subscribed to the existing subscription. For example, if all components subscribe to an AWS Credential and that Credential is not in the working set, they will remain subscribed to the original AWS Credential.
:::

### Dry Run Mode

Templates support a dry-run mode that previews what changes would be made without actually creating or modifying any components. This is useful for:

- Verifying search queries match the expected components
- Testing input variable parsing and validation
- Previewing component name transformations
- Checking attribute changes before applying them

<DocTabs tabs="CLI">
<TabPanel value="CLI">

To run a template in dry-run mode:

```shellscript [Dry Run]
$ si template run vpc-pattern.ts --key test --dry-run
✨ info    si              Loading Template: "file:///home/toddhoward/templates/vpc-pattern.ts"
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
✨ info    si              Loaded baseline component "AWS::EC2::EIP" "demo-eip-natgw-1c" (1/29)
✨ info    si              Loaded baseline component "AWS::EC2::EIP" "demo-eip-natgw-1b" (2/29)
✨ info    si              Loaded baseline component "AWS::EC2::EIP" "demo-eip-natgw-1a" (3/29)
✨ info    si              Loaded baseline component "AWS::EC2::InternetGateway" "demo-igw" (4/29)
✨ info    si              Loaded baseline component "AWS::EC2::NatGateway" "demo-natgw-1a" (5/29)
✨ info    si              Loaded baseline component "AWS::EC2::NatGateway" "demo-natgw-1b" (6/29)
✨ info    si              Loaded baseline component "AWS::EC2::NatGateway" "demo-natgw-1c" (7/29)
✨ info    si              Loaded baseline component "AWS::EC2::Route" "demo-public-route-igw" (8/29)
✨ info    si              Loaded baseline component "AWS::EC2::Route" "demo-private-route-natgw-1a" (9/29)
✨ info    si              Loaded baseline component "AWS::EC2::Route" "demo-private-route-natgw-1c" (10/29)
✨ info    si              Loaded baseline component "AWS::EC2::Route" "demo-private-route-natgw-1b" (11/29)
✨ info    si              Loaded baseline component "AWS::EC2::RouteTable" "demo-private-rtb-1a" (12/29)
✨ info    si              Loaded baseline component "AWS::EC2::RouteTable" "demo-private-rtb-1c" (13/29)
✨ info    si              Loaded baseline component "AWS::EC2::RouteTable" "demo-private-rtb-1b" (14/29)
✨ info    si              Loaded baseline component "AWS::EC2::RouteTable" "demo-public-rtb" (15/29)
✨ info    si              Loaded baseline component "AWS::EC2::Subnet" "demo-public-subnet-1a" (16/29)
✨ info    si              Loaded baseline component "AWS::EC2::Subnet" "demo-private-subnet-1a" (17/29)
✨ info    si              Loaded baseline component "AWS::EC2::Subnet" "demo-public-subnet-1b" (18/29)
✨ info    si              Loaded baseline component "AWS::EC2::Subnet" "demo-private-subnet-1c" (19/29)
✨ info    si              Loaded baseline component "AWS::EC2::Subnet" "demo-private-subnet-1b" (20/29)
✨ info    si              Loaded baseline component "AWS::EC2::Subnet" "demo-public-subnet-1c" (21/29)
✨ info    si              Loaded baseline component "AWS::EC2::SubnetRouteTableAssociation" "demo-public-subnet-1a-rtb-assoc" (22/29)
✨ info    si              Loaded baseline component "AWS::EC2::SubnetRouteTableAssociation" "demo-public-subnet-1c-rtb-assoc" (23/29)
✨ info    si              Loaded baseline component "AWS::EC2::SubnetRouteTableAssociation" "demo-private-subnet-1c-rtb-assoc" (24/29)
✨ info    si              Loaded baseline component "AWS::EC2::SubnetRouteTableAssociation" "demo-public-subnet-1b-rtb-assoc" (25/29)
✨ info    si              Loaded baseline component "AWS::EC2::SubnetRouteTableAssociation" "demo-private-subnet-1a-rtb-assoc" (26/29)
✨ info    si              Loaded baseline component "AWS::EC2::SubnetRouteTableAssociation" "demo-private-subnet-1b-rtb-assoc" (27/29)
✨ info    si              Loaded baseline component "AWS::EC2::VPC" "demo-vpc" (28/29)
✨ info    si              Loaded baseline component "AWS::EC2::VPCGatewayAttachment" "demo-igw-attachment" (29/29)
✨ info    si              Built baseline with 29 components from search
✨ info    si              Initializing working set: 29 components
✨ info    si              Getting or creating change set: "vpc-pattern-test"
✨ info    si              Found 0 existing components
✨ info    si              Computing delta
✨ info    si              Pending changes: 29 creates, 0 updates, 0 deletes
✨ info    si              Dry Run: Creating "AWS::EC2::EIP" "demo-eip-natgw-1c" (1/29)
✨ info    si              Dry Run: Setting attributes on "demo-eip-natgw-1c": { "/si/name": "demo-eip-natgw-1c",
                             "/si/type": "component",
                             "/domain/Domain": "vpc",
                             "/domain/extra/Region":
                              { "$source": { component: "01K3SS2GHZJ2Z548EY6VDVVW69", path: "/domain/region" } },
                             "/secrets/AWS Credential":
                              { "$source": { component: "01K3SS2G957T335HDHFR8VP86Q", path: "/secrets/AWS Credential" } } }
✨ info    si              Dry Run: Creating "AWS::EC2::EIP" "demo-eip-natgw-1b" (2/29)
✨ info    si              Dry Run: Setting attributes on "demo-eip-natgw-1b": { "/si/name": "demo-eip-natgw-1b",
                             "/si/type": "component",
                             "/domain/Domain": "vpc",
                             "/domain/extra/Region":
                              { "$source": { component: "01K3SS2GHZJ2Z548EY6VDVVW69", path: "/domain/region" } },
                             "/secrets/AWS Credential":
                              { "$source": { component: "01K3SS2G957T335HDHFR8VP86Q", path: "/secrets/AWS Credential" } } }
✨ info    si              Dry Run: Creating "AWS::EC2::EIP" "demo-eip-natgw-1a" (3/29)
...
✨ info    si              Dry run complete: 29 creates, 0 updates, 0 deletes
~/templates (sibook)
```

Dry-run mode executes the template logic, including search, input validation, name patterns, and transformations, but stops before creating the change set or making any modifications to the workspace.

</TabPanel>
</DocTabs>

### Invocation Key

Each template run requires an invocation key to be passed with the `--key` parameter. This value is used to correlate a particular invocation of the template to the components it creates or updates. This is how we enable idempotency over subsequent invocations of the template. By allowing you to specify the key, we also enable you to run the same template multiple times against the same workspace.

## Baseline Components

Baseline components can be defined in two ways: either through dynamic search or as static YAML.

### Dynamic Search

Specify the baseline components for your template with one or more search strings. The results of all searches (deduplicated) will become the working set.

```typescript
import { TemplateContext } from "jsr:@systeminit/si";

export default async function (ctx: TemplateContext) {
  ctx.search([ // [!code focus:4]
    'schema:"AWS::EC2::VPC"',
    'schema:"AWS::EC2::Subnet"',
  ]);
}
```

:::tip
The search syntax in System Initiative can express very complex boolean logic - you most likely only need one search query for almost all use cases. Supporting multiple search strings is provided for convenience when composing complex templates, breaking up the search logic as needed.
:::

### YAML Cache

To make templates re-usable even without access to the workspaces the baseline components exist in, you can create a "baseline cache" file. This stores the results of your baseline search (and some information about the schemas they use) as YAML.

Given a template like the one we defined above for Dynamic Search, we would cache all the VPC and Subnet components in our workspace.

<DocTabs tabs="CLI">
<TabPanel value="CLI">

To create a baseline YAML cache with the CLI:

```shellscript [Create a Template]
$ si template run ./vpc-and-subnet.ts --key cache --cache-baseline vpc-and-subnet-baseline.yaml --cache-baseline-only
✨ info    si              Loading Template: "file:///home/adam/src/template-test/vpc-and-subnet.ts"
✨ info    si              Building baseline with search strings: [ 'schema:"AWS::EC2::VPC"', 'schema:"AWS::EC2::Subnet"' ]
✨ info    si              Found 3 unique components from search
✨ info    si              Loaded baseline component "AWS::EC2::VPC" "demo-vpc" (1/3)
✨ info    si              Loaded baseline component "AWS::EC2::Subnet" "demo-subnet-1" (2/3)
✨ info    si              Loaded baseline component "AWS::EC2::Subnet" "demo-subnet-2" (3/3)
✨ info    si              Built baseline with 3 components from search
✨ info    si              Caching baseline to "vpc-and-subnet-baseline.yaml"
✨ info    si              Cached 3 components and 2 schemas to "vpc-and-subnet-baseline.yaml"
✨ info    si              Baseline cache written successfully. Exiting.
```

</TabPanel>
</DocTabs>

The resulting YAML file contains all the information about your baseline components.

:::tip
Make a cache of your baseline frequently, and store it alongside the template file in source control. This will allow you to recreate the infrastructure in your workspace exactly as it was at a given point in time!
:::

<DocTabs tabs="CLI">
<TabPanel value="CLI">

To utilize this cache when running a template, specify the `--baseline` option to `template run`:

```shellscript [Create a Template]
$ si template run ./vpc-and-subnet.ts --key rebuild --baseline vpc-and-subnet-baseline.yaml
✨ info    si              Loading Template: "file:///home/adam/src/template-test/vpc-and-subnet.ts"
✨ info    si              Loading baseline data from "vpc-and-subnet-baseline.yaml"
✨ info    si              Initializing working set: 3 components
✨ info    si              Getting or creating change set: "vpc-and-subnet-rebuild"
✨ info    si              Found 0 existing components
✨ info    si              Computing delta
✨ info    si              Pending changes: 3 creates, 0 updates, 0 deletes
✨ info    si              Creating "AWS::EC2::VPC" "demo-vpc" (1/3)
✨ info    si              Creating "AWS::EC2::Subnet" "demo-subnet-2" (2/3)
✨ info    si              Creating "AWS::EC2::Subnet" "demo-subnet-1" (3/3)
✨ info    si              Execution complete: 3 succeeded, 0 failed
```
</TabPanel>
</DocTabs>

## Input Variables

Templates can specify input data that can be used as variables when transforming the working set.

:::tip
If you are familiar with Terraform or Open Tofu, template inputs are the equivalent of input variables.
:::

### Input Schema

You specify the input schema using the [zod schema language](https://zod.dev/).

```typescript [vpc-and-subnet.ts]
import { TemplateContext } from "jsr:@systeminit/si";
import { z } from "npm:zod@4"; // [!code focus]

export default async function (ctx: TemplateContext) {
  const inputSchema = z.object({ // [!code focus:10]
    environment: z.string().default("dev"),
    region: z.enum([ "us-east-1", "us-east-2" ]),
    vpcAttributes: z.object({
      vpcCount: z.number(),
    }),
  });
  type Inputs = z.infer<typeof inputSchema>;

  ctx.inputs(inputSchema)

  ctx.search([
    'schema:"AWS::EC2::VPC"',
    'schema:"AWS::EC2::Subnet"',
  ]);
}
```

This code defines an input schema that is an object with 3 keys: environment, region, and vpcAttributes. The environment value is a string that defaults to "dev", the region is either us-east-1 or us-east-2, and vpcAttributes is itself an object with a single vpcCount key, whose value is a number.

:::tip
Wondering what the `type Inputs = z.infer<typeof inputSchema>` line is all about? It's a little bit of TypeScript magic that creates a TypeScript type from the schema you defined, so that we can use it later to have your editor provide intellisense on your input data!
:::

There is a helper export you can use if you want to specify the details of a subscription as an input, to be later consumed by the [ensure attribute helpers](#setting-subscriptions) named `SubscriptionInput`. To use it:


```typescript [vpc-and-subnet.ts]
import { SubscriptionInput, TemplateContext } from "jsr:@systeminit/si"; // [!code focus]
import { z } from "npm:zod@4";

export default async function (ctx: TemplateContext) {
  const inputSchema = z.object({ // [!code focus]
    environment: z.string().default("dev"),
    region: SubscriptionInput, // [!code focus]
    vpcAttributes: z.object({
      vpcCount: z.number(),
    }),
  }); // [!code focus]
  type Inputs = z.infer<typeof inputSchema>;

  ctx.inputs(inputSchema)

  ctx.search([
    'schema:"AWS::EC2::VPC"',
    'schema:"AWS::EC2::Subnet"',
  ]);
}
```

### YAML Input Files

To specify the inputs to the template, write a YAML file that matches your schema. The best practice is to have the file end with '-input.yaml', such as 'vpc-and-subnet-input.yaml'.

:::code-group

```yaml [Region Enum]
environment: prod
region: us-east-1
vpcAttributes:
  vpcCount: 2
```

```yaml [Region Subscription]
environment: prod
region:
  kind: "search"
  query: 'name:"standard region"'
  path: "/domain/region"
vpcAttributes:
  vpcCount: 2
```

:::

To specify your input values when running the template, add the `--input` option:

```shellscript [template run with inputs]
$ si template run ./vpc-and-subnet.ts --key rebuild --input ./vpc-and-subnet-inputs.yaml
✨ info    si              Loading Template: "file:///home/adam/src/template-test/vpc-and-subnet.ts"
✨ info    si              Loading input data from: "./vpc-and-subnet-inputs.yaml"
✨ info    si              Building baseline with search strings: [ 'schema:"AWS::EC2::VPC"', 'schema:"AWS::EC2::Subnet"' ]
✨ info    si              Found 3 unique components from search
✨ info    si              Loaded baseline component "AWS::EC2::VPC" "demo-vpc" (1/3)
✨ info    si              Loaded baseline component "AWS::EC2::Subnet" "demo-subnet-1" (2/3)
✨ info    si              Loaded baseline component "AWS::EC2::Subnet" "demo-subnet-2" (3/3)
✨ info    si              Built baseline with 3 components from search
✨ info    si              Initializing working set: 3 components
✨ info    si              Getting or creating change set: "vpc-and-subnet-rebuild"
✨ info    si              Found 0 existing components
✨ info    si              Computing delta
✨ info    si              Pending changes: 3 creates, 0 updates, 0 deletes
✨ info    si              Creating "AWS::EC2::VPC" "demo-vpc" (1/3)
✨ info    si              Creating "AWS::EC2::Subnet" "demo-subnet-1" (2/3)
✨ info    si              Creating "AWS::EC2::Subnet" "demo-subnet-2" (3/3)
✨ info    si              Execution complete: 3 succeeded, 0 failed
```

### Referencing Inputs

You can reference the input data either as the second argument to the transformation function or as the second argument to the rename function. Both are demonstrated in the next section.

:::tip
You specify the input schema in your template file, but the inputs themselves are not evaluated until your template is executed. That is why the input data is not available outside the transformation function or replacement patterns.
:::

## Renaming Components

The most common change a template makes is to rename components to make them distinct. You can do this by specifying one or more name patterns, which will be applied to the working set in order.

```typescript [namePattern]
export default async function (ctx: TemplateContext) {
  const inputSchema = z.object({
    environment: z.string().default("prod"),
    region: z.enum(["us-east-1", "us-east-2"]),
    vpcAttributes: z.object({
      vpcCount: z.number(),
    }),
  });
  type Inputs = z.infer<typeof inputSchema>;

  ctx.inputs(inputSchema);

  ctx.search([
    'schema:"AWS::EC2::VPC"',
    'schema:"AWS::EC2::Subnet"',
  ]);

  ctx.namePattern([ // [!code focus:3]
    { pattern: /demo-(.+)/, replacement: "prod-$1" },
  ]);
}
```

A name pattern consists of two arguments: the initial pattern and the replacement pattern.

The initial pattern is a [JavaScript regular expression](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Regular_expressions). The replacement is a [JavaScript string replacement pattern](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/replace#specifying_a_string_as_the_replacement).

In the above example, it will rename all components from "demo-whatever" to "prod-whatever".

Each name pattern you provide will be evaluated against the entire working set, in order. This means you can have multiple patterns that rename the same component.

#### Using Input Variables in Replacement Patterns

To use input variables in replacement patterns, use [EJS string templating syntax](https://ejs.co/#docs) in the replacement pattern. Assuming a template with the following input schema and name pattern:

```typescript [namePattern]
export default async function (ctx: TemplateContext) {
  const inputSchema = z.object({
    environment: z.string().default("prod"), // [!code focus]
    region: z.enum(["us-east-1", "us-east-2"]),
    vpcAttributes: z.object({
      vpcCount: z.number(),
    }),
  });
  type Inputs = z.infer<typeof inputSchema>;

  ctx.inputs(inputSchema);

  ctx.search([
    'schema:"AWS::EC2::VPC"',
    'schema:"AWS::EC2::Subnet"',
  ]);

  ctx.namePattern([ // [!code focus:3]
    { pattern: /demo-(.+)/, replacement: "<%= inputs.environment %>-$1" },
  ]);
}
```

:::tip
While you can write complex expressions with EJS, typically all you will need is the syntax above, which inserts a variable into the string.
:::

With the following input YAML:

```yaml [vpc-and-subnet-input.yaml]
environment: prod
region: us-east-1
vpcAttributes:
  vpcCount: 2
```

Results in the following output:

```shellscript
$ si template run ./vpc-and-subnet.ts --key rebuild --input ./vpc-and-subnet-inputs.yaml                                   6s
✨ info    si              Loading Template: "file:///home/adam/src/template-test/vpc-and-subnet.ts"
✨ info    si              Loading input data from: "./vpc-and-subnet-inputs.yaml"
✨ info    si              Building baseline with search strings: [ 'schema:"AWS::EC2::VPC"', 'schema:"AWS::EC2::Subnet"' ]
✨ info    si              Found 3 unique components from search
✨ info    si              Loaded baseline component "AWS::EC2::VPC" "demo-vpc" (1/3)
✨ info    si              Loaded baseline component "AWS::EC2::Subnet" "demo-subnet-1" (2/3)
✨ info    si              Loaded baseline component "AWS::EC2::Subnet" "demo-subnet-2" (3/3)
✨ info    si              Built baseline with 3 components from search
✨ info    si              Initializing working set: 3 components
✨ info    si              Applying pattern 1/1: "demo-(.+)" -> "prod-$1"
✨ info    si              Getting or creating change set: "vpc-and-subnet-rebuild"
✨ info    si              Found 0 existing components
✨ info    si              Computing delta
✨ info    si              Pending changes: 3 creates, 0 updates, 0 deletes
✨ info    si              Creating "AWS::EC2::VPC" "prod-vpc" (1/3)
✨ info    si              Creating "AWS::EC2::Subnet" "prod-subnet-1" (2/3)
✨ info    si              Creating "AWS::EC2::Subnet" "prod-subnet-2" (3/3)
✨ info    si              Execution complete: 3 succeeded, 0 failed
```

The components have all been renamed according to the 'environment' input variable.

## Transforming the Working Set

Transforming the working set lets you manipulate component attributes, create new components, clone existing components, or remove components from the set. This is done through adding a transformation function:

```typescript
import { TemplateContext } from "jsr:@systeminit/si";
import { z } from "npm:zod@4";

export default async function (ctx: TemplateContext) {
  const inputSchema = z.object({
    environment: z.string().default("prod"),
    region: z.enum(["us-east-1", "us-east-2"]),
    vpcAttributes: z.object({
      vpcCount: z.number(),
    }),
  });
  type Inputs = z.infer<typeof inputSchema>; // [!code focus]

  ctx.inputs(inputSchema);

  ctx.search([
    'schema:"AWS::EC2::VPC"',
    'schema:"AWS::EC2::Subnet"',
  ]);

  ctx.namePattern([
    { pattern: /demo-(.+)/, replacement: "<%= inputs.environment %>-$1" },
  ]);

  ctx.transform(async (workingSet, inputs) => { // [!code focus:5]
    inputs = inputs as Inputs;

    return workingSet;
  });
}
```

The function takes an async callback function, whose arguments are the `workingSet` and your `inputs`. The function must return a `workingSet` (if you do not, your transformation function may not work as expected.)

:::info
The `inputs = inputs as Inputs` line is a bit of TypeScript magic that enables intellisense for your input variables.
:::

### Finding components in the working set

To find components in the working set, you can use the [find function](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/find):

```typescript
const subnet = workingSet.find((component) => component.name === "prod-subnet-1");
if (subnet) {
  // transform it
}
```

You can also iterate over the components in the working set, and use conditional logic:

```typescript
for (const component of workingSet) {
  if (component.name === "prod-subnet-1") {
    // transform it
  }
}
```

### Changing Attributes

#### ensureAttribute

Once you have found the component you want to set the attributes of, you can set its attributes with the `ensureAttribute` helper. It takes four arguments:

1. The component to update
1. The attribute path to set
1. The value to set the attribute to, or the subscription to create
1. An optional object that modifies the behavior of the helper

```typescript
const subnet = workingSet.find((component) => component.name === "prod-subnet-1");
if (subnet) {
  await ctx.ensureAttribute(
    subnet,
    "/domain/EnableDns64",
    true,
  )
}
```

You can set the attribute to any value that fits the schema: boolean, string, number, object, map, array, etc.

:::tip
The attribute path syntax automatically creates nested objects and maps entries as needed. You can either set the whole object as a value or set individual properties.
:::

##### Setting Subscriptions

You can set a subscription to another component's attribute through searching for the component to subscribe to or by referencing it directly by name or ID.

:::code-group

```typescript [Search]
await ctx.ensureAttribute(
  component,
  "/domain/extra/Region",
  {
    kind: "search",
    query: 'name:"demo region"',
    path: "/domain/region",
  },
);
```

```typescript [By Name]
await ctx.ensureAttribute(
  component,
  "/domain/extra/Region",
  {
    kind: "$source",
    component: "demo region",
    path: "/domain/region",
  },
);
```

```typescript [By Component ID]
await ctx.ensureAttribute(
  component,
  "/domain/extra/Region",
  {
    kind: "$source",
    component: "01K2YVY4WE8KBM01H05R74RKX8", // Component ID
    path: "/domain/region",
  },
);
```

:::

The [public API](./public-api.md) supports a low-level API for setting subscriptions using `$source` syntax, which can also be used.

```typescript
await ctx.ensureAttribute(
  component,
  "/domain/extra/Region",
  {
    $source: {
      path: "/domain/region",
    }
  }
);
```

:::tip
The entire ensure* family of helper functions supports the same subscription syntax for values.
:::

##### skipIfMissing

The `ensureAttribute` helper will only set the value on the component if needed (it is idempotent), and will throw an error if the attribute you are trying to set does not exist in the component's schema. For convenience, you can disable this check with an optional `skipIfMissing` argument:

```typescript
for (const component of workingSet) {
  ctx.ensureAttribute(
    component,
    "/domain/EnableDns64",
    true,
    { skipIfMissing: true }
  );
}
```

In this example, we iterate over every component in the working set, and set the "/domain/EnableDns64" attribute to true *only if that attribute exists on the component's schema*, otherwise we do nothing.

:::tip
All the `ensure*` family of helpers support the `skipIfMissing` option.
:::

#### ensureAttributeMissing

If you want to ensure an attribute is not set, use `ensureAttributeMissing`:

```typescript
await ctx.ensureAttributeMissing(
  component,
  "/domain/EnableDns64",
)
```

It takes 3 arguments:

1. The component to update
1. The attribute path to ensure is missing
1. The optional object to modify the behavior of the helper

Ensuring an attribute is missing will delete the values entirely from the component's attributes. It is idempotent - if the attribute is already missing, nothing is done.

#### ensureArrayAttribute

If you need to make sure an array has an entry with a specific value, use `ensureArrayAttribute`. It supports finding and replacing specific values, adding new values if none are present, and supports the subscription syntax for values.

##### Scalar Values

To ensure an array contains a scalar value, such as a string, number, or boolean:

```typescript
await ctx.ensureArrayAttribute(
  component,
  "/domain/Ports",
  (e) => e.value === 8080,
  8080
);
```

It takes five arguments:

1. The component to update
1. The attribute path to the array
1. A match function that determines if a given entry in the array should be changed
1. The value to set in all matching positions
1. The optional object to modify the behavior of the helper

In the above example, if there is an entry with the value `8080`, it will not be changed. If there is no value that matches `8080`, then a new entry in the array will be added with `8080`.

To update a specific value to a new value, change the match function:

```typescript
await ctx.ensureArrayAttribute(
  component,
  "/domain/Ports",
  (e) => e.value === 80,
  8080
);
```

In this case, it would change any entry whose value is `80` to `8080`.

##### Object Values

Arrays of objects are a common pattern in many APIs. Given a structure like:

```json
{
  "/domain/Tags": [
    { Key: "Name", Value: "demo-subnet-1" },
    { Key: "Cost Center", Value: "operations" },
  ],
}
```

We can update the Name entry with the following code:

```typescript
await c.ensureArrayAttribute(
  component,
  "/domain/Tags",
  (e) => e.subpath === "Key" && e.value === "Name",
  { Key: "Name", Value: "demo-subnet-awesome" }
);
```

This would change the 'Value' of the object whose 'Key' is 'Name' to 'demo-subnet-awesome', rather than 'demo-subnet-1'. If there is no entry with 'Key: Name', then one will be created.

##### Partial object merging

If you provide a partial object for the value, it will be merged with any existing elements. This allows you to update an individual element of an object without changing the other properties:

```typescript
await c.ensureArrayAttribute(
  component,
  "/domain/Tags",
  (e) => e.subpath === "Key" && e.value === "Name",
  { Value: "demo-subnet-awesome" }
);
```

##### The match function

The match function gets each element of the array (by convention, the variable 'e'), and if it returns true, it will update that element.

The element object has the following properties:

- *subpath*: Each path beneath the entry; for example, "Key" or "Cost Center"
- *value*: The current value for this subpath
- *fullPath*: The complete attribute path (/domain/Tags/0/Key)
- *index*: The current array index (0 or 1, for example)

#### ensureArrayAttributeMissing

The `ensureArrayAttributeMissing` helper deletes array elements (or specific properties) that match your criteria, and then reindexes the array (to avoid sparse arrays).

##### Delete entire array elements

Given a structure like this:

```json
{
  "/domain/Ports": [
    "80",
    "8080",
    "443",
  ],
}
```

We can remove port 8080 as follows:

```typescript
await c.ensureArrayAttributeMissing(
  component,
  "/domain/Ports",
  (e) => e.value === "8080"
);
```

Which will result in an array with `[ "80", "443" ]`.

It takes five arguments:

1. The component to update
1. The attribute path to the array
1. A match function that determines if a given entry in the array should be removed
1. An optional array of keys that should be deleted
1. The optional object to modify the behavior of the helper

##### Deleting specific properties of an array entry

Occasionally, you will want to delete only particular properties within an array entry. In this case, you can specify those as the fourth argument.

Given a structure like this:

```json
{
  "/domain/Tags": [
    { Key: "Name", useInTemplate: true, Value: "Crow" },
    { Key: "Cost Center", useInTemplate: true, Value: "operations" },
  ],
}
```


```typescript
await c.ensureArrayAttributeMissing(
  component,
  "/domain/Tags",
  (e) => e.subpath === "Key" && e.value === "Name",
  ["useInTemplate"]
);
```

Will remove only `useInTemplate: true` from the matching entry object.

### Creating new components

You can create new components with the `newComponent` helper.

```typescript
c.transform(async (workingSet, inputs) => {
  const newServer = await c.newComponent(
    "AWS::EC2::Subnet",
    "public-subnet-1",
    {
      "/domain/CidrBlock": "10.0.1.0/24",
      "/domain/MapPublicIpOnLaunch": true,
      "/domain/Tags/0/Key": "Name",
      "/domain/Tags/0/Value": "public-subnet-1",
    }
  );
  workingSet.push(newServer);

  return workingSet;
});
```

There are 3 arguments:

1. The schema of the component you want to create
1. The name of the component
1. The attributes you want to set, expressed as attribute paths.

:::warning
You must always push the variable containing your new component on to the workingSet if you want it to be created!
:::

### Copying existing components

A common use case in templating is creating a variable number of components based on an example. Do that with the `copyComponent` helper:

```typescript
ctx.transform(async (workingSet, inputs) => {
  const subnetExample = workingSet.find(c => c.name === "subnet-example");
  for (let i = 1; i <= 5; i++) {
    const newSubnet = ctx.copyComponent(subnetExample, `subnet-${i}`);
    workingSet.push(newSubnet);
  }

  return workingSet;
});

```

There are 2 arguments:

1. The component to copy
1. The name of the new component

A typical pattern is to use the `ensure` helper functions to configure your newly cloned component appropriately.

:::warning
You must always push the variable containing your new component onto the workingSet if you want it to be created!
:::

## Specifying the Template Name

By default, every template will be named after the file it is stored in, minus the `.ts` extension. To set the name directly:

```typescript
import { TemplateContext } from "jsr:@systeminit/si";
import { z } from "npm:zod@4";

export default async function (ctx: TemplateContext) {
  ctx.name("new-name");
}
```

To retrieve the name from a template file called `new-vpc.ts`:

```typescript
import { TemplateContext } from "jsr:@systeminit/si";
import { z } from "npm:zod@4";

export default async function (ctx: TemplateContext) {
  const currentName = ctx.name(); // returns "new-vpc" for "new-vpc.ts"
}
```

## Specifying the Change Set

By default, the template will create a change set that is the combination of the template's name and its invocation key (the --key argument).

For example, if the template file is named 'new-vpc.ts' and `--key rebuild` is passed to `template run`, the change set would be *new-vpc-rebuild*.

To set the change set name directly:

```typescript
import { TemplateContext } from "jsr:@systeminit/si";
import { z } from "npm:zod@4";

export default async function (ctx: TemplateContext) {
  ctx.changeSet("create-new-stuff");
}
```

To retrieve the change set name:

```typescript
import { TemplateContext } from "jsr:@systeminit/si";
import { z } from "npm:zod@4";

export default async function (ctx: TemplateContext) {
  const changeSetName = ctx.changeSet(); // returns "new-vpc-rebuild"
}
```

## Getting the Invocation Key

You can get the invocation key with:

```typescript
import { TemplateContext } from "jsr:@systeminit/si";
import { z } from "npm:zod@4";

export default async function (ctx: TemplateContext) {
  const invocationKey = ctx.invocationKey(); // returns "rebuild"
}
```

## Using NPM or JSR Libraries

Templates are invoked with the [Deno TypeScript runtime](https://deno.com). They support dynamically importing any module from the [NPM](https://npmjs.com) or [JSR](https://jsr.io) package registries.

```typescript
import { TemplateContext } from "jsr:@systeminit/si";
import { z } from "npm:zod@4";
```

The first line imports the [@systeminit/si](https://jsr.io/@systeminit/si) library, which defines our templating syntax, from JSR.

The second line imports the [zod](https://jsr.io/@systeminit/si) schema validation library from NPM. It also ensures that we get the most recent release of version 4 of the library.

You can see the [full syntax and range of options in the Deno documentation](https://docs.deno.com/runtime/fundamentals/modules/#importing-third-party-modules-and-libraries).

:::tip
This means you can use the entirety of the JavaScript ecosystem to create truly dynamic template solutions. Call out to third-party APIs, do complex network address math, or integrate with your internal systems.
:::

## LSP Support

Follow the instructions to [set up your environment for Deno development](https://docs.deno.com/runtime/getting_started/setup_your_environment/) to enable the Deno LSP server for your preferred editor. This will provide inline documentation for every template helper, and intellisense for the typescript types as you develop.

