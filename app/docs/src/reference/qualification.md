# Qualification Functions

Qualification functions are a kind of [function](./function.md) that validates
[component](./components.md) configuration by checking whether the component
meets specific requirements or constraints. They provide real-time feedback
about configuration issues, policy compliance, or resource availability before
you attempt to create or modify infrastructure.

## How Qualification Functions Work

Qualification functions examine a component's configuration, generated code, and
resource state to determine if it meets certain criteria. They return one of
three results:

- **Success**: The component passes validation
- **Warning**: The component has issues but isn't critically broken
- **Failure**: The component has critical problems that should be addressed

Qualification results appear as colored indicators on components in the web UI,
giving you immediate visual feedback about configuration quality.

## When Qualification Functions Run

Qualification functions execute automatically:

- When a component is created
- When component properties change
- After code generation functions complete (if the qualification depends on
  generated code)
- On HEAD, when you explicitly run qualifications to check the actual resource state

Unlike most functions, qualifications can run on HEAD without going through a
change set, since they only read state and don't modify resources.

## Qualification Function Arguments

Qualification functions take a `component` argument with these properties:

- `code`: Generated code available as a map keyed by code generation function
  name
- `domain`: The component's domain properties
- `resource`: Resource information from previous actions
- `deleted_at`: Timestamp string if the component has been deleted

This gives qualifications access to all component and resource data, allowing
them to validate the configuration comprehensively, both for the model and the real
world.

## Qualification Function Return Value

Qualifications return an object with two properties:

```typescript
return {
  result: "success", // or "warning" or "failure"
  message: "Configuration is valid",
};
```

The `result` determines the visual indicator shown on the component, while the
`message` provides details about the reasons for the result.

## Common Use Cases

Qualification functions are commonly used for:

- **Policy validation**: Checking that resources comply with organizational
  policies (required tags, naming conventions, etc.)
- **Resource availability**: Verifying that referenced resources exist (AMI IDs,
  subnet IDs, etc.)
- **Configuration validation**: Ensuring generated code is syntactically valid
- **Cloud provider checks**: Running provider-specific validators (AWS IAM
  Policy Simulator, etc.)
- **Dependency verification**: Checking that required resources are present
- **Security compliance**: Validating encryption settings, access controls, or
  other security requirements
- **Cost estimation**: Warning about configurations that may be expensive

## Qualifications vs Validations

Qualifications differ from simple property validations:

- **Property validations**: Check individual properties in isolation (type
  checking, required fields, etc.)
- **Qualifications**: Examine the entire component configuration, generated
  code, and external state to validate holistically

Qualifications can make external API calls, run command-line tools, and perform
complex logic that simple property validations cannot.

## Real-World Validation Examples

Qualifications can perform sophisticated checks:

- **AWS IAM Policy Simulator**: Validate that IAM policies grant the intended
  permissions
- **Docker Registry Check**: Verify that a specified container image exists and
  is accessible
- **Butane Validation**: Ensure that Butane configuration generates valid
  Ignition files
- **DNS Resolution**: Check that domain names resolve correctly
- **API Endpoint Testing**: Verify that endpoints are reachable

## Qualifications on HEAD

Qualifications are unique in that they can run directly on HEAD without a change
set, since they only read state. This allows you to:

- Check if resources still exist upstream
- Verify that external dependencies are still valid
- Monitor compliance over time
- Detect configuration drift

## See Also

For detailed examples and technical implementation details, see the
[Qualification Function Examples](/reference/function#qualification-function-examples)
section in the Functions Reference.
