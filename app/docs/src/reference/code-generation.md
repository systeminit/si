# Code Generation Functions

Code Generation functions are a kind of [function](./function.md) that generates
code or configuration from [component](./components.md) data. They transform
your component's properties into the exact format needed by external tools and
APIs, making it easy to see what will be sent to your cloud provider before
actions execute.

## How Code Generation Functions Work

Code Generation functions take a component's current configuration and generate
formatted output - typically JSON, YAML, or other structured formats. The
generated code:

- Appears in the Code Gen section of the component details page
- Can be accessed by action functions when creating or updating resources
- Is available in other functions as part of the component's code property
- Updates automatically when the component's properties change

This provides transparency into exactly what configuration will be sent to
external services and allows you to validate it before taking action.

## When Code Generation Functions Run

Code Generation functions execute automatically:

- When a component is first created
- When component properties that the function depends on change
- Before actions that use the generated code execute

The generated code is stored with the component, so action functions can access
it without re-running the code generation.

## Code Generation Function Arguments

Code Generation functions receive a single `component` argument with these
properties:

- `domain`: The component's domain properties (your configuration)
- `resource`: The resource information from previous actions (if any)
- `deleted_at`: A timestamp if the component has been deleted

This gives the function access to both the intended configuration and any
current resource state.

## Code Generation Function Return Value

Code Generation functions return an object with two properties:

```typescript
{
    format: "json",  // or "yaml", "toml", etc.
    code: '{ "poop": "canoe" }',  // the generated code as a string
}
```

The `format` indicates how the code should be displayed in the UI, while `code`
contains the actual generated output.

## Accessing Generated Code

Other functions can access generated code through the component's `code`
property. For example, an action function might use:

```typescript
const code = component.properties.code?.["si:genericAwsCreate"]?.code;
```

The code is stored in a map keyed by the code generation function's name.

## Common Use Cases

Code Generation functions are commonly used for:

- **Cloud provider API payloads**: Generating JSON for AWS CLI
  `--cli-input-json` parameters
- **Infrastructure as Code**: Creating Terraform, CloudFormation, or other IaC
  formats
- **Configuration files**: Generating config files for services (nginx.conf,
  systemd units, etc.)
- **API request bodies**: Formatting data for REST API calls
- **Validation input**: Providing formatted input for qualification functions
- **External tool input**: Creating input files for command-line tools

## Code Generation with External Tools

Code Generation functions can use external tools to format or validate their
output. For example, you might use `butane` to validate and format Ignition
configuration, or `jq` to pretty-print JSON.

The generated code can be piped through any available command-line tool to
ensure it meets the required format.

## Code Generation and Actions

Code Generation functions work hand-in-hand with actions:

1. You configure a component's properties
2. Code Generation functions create the API payload
3. You review the generated code in the Code tab
4. When you run an action, it uses the pre-generated code
5. This ensures you see exactly what will be sent before it happens

This transparency is a key part of System Initiative's approach to
infrastructure management.

## See Also

For detailed examples and technical implementation details, see the
[Code Generation Function Examples](/reference/function#code-generation-function-examples)
section in the Functions Reference.
