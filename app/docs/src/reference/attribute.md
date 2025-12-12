# Attribute Functions

Attribute functions are a kind of [function](./function.md) used to set
properties on [components](./components.md) using data from other properties on
the same or other components. They are the primary mechanism for transforming
data within a component.

## How Attribute Functions Work

Attribute functions execute automatically whenever their input data changes.
They take data from one or more sources (such as other component properties) and
compute a result that is stored in a specific location within the component's
property tree.

Each attribute function has:

- **Arguments**: The input data sources the function needs to perform its
  computation
- **Bindings**: The configuration that specifies where the function's output
  will be stored and where each argument's data comes from
- **Output location**: A path in the component's property tree where the
  function's return value is stored

## When Attribute Functions Run

Attribute functions execute automatically in these scenarios:

- When a component is first created
- When a subscription receives new data
- When a property that the function depends on changes
- After other attribute functions that this function depends on complete

System Initiative determines the correct execution order based on the
dependencies between attribute functions.

## Attribute Function Arguments

Attribute functions receive a single `input` object as their argument. The
properties on this object are determined by the Arguments section in the
function's metadata panel.

Each argument has:

- **Name**: Used as the property name on the `input` object
- **Type**: One of Any, Array, Boolean, Integer, JSON, Map, Object, or String

These types map directly to their TypeScript equivalents and align with schema
property kinds.

## Attribute Function Bindings

Bindings connect the function's arguments to their data sources and specify
where the output goes.

For each attribute function, you configure:

- **Output location**: A path (like `/domain/snack`) where the function's return
  value will be stored
- **Argument sources**: For each function argument, specify whether its data
  comes from a subscription or another property in the same component

For example, an attribute function that writes to the `snack` attribute from the
value of the `Yummy` property subscription would have:

- A function argument named `yummy` with its source set to the `Yummy` property
  via a subscription
- An output location of `/domain/snack`

Bindings are configured in the `Bindings` sub-panel of the function's metadata.

## Common Use Cases

Attribute functions are commonly used for:

- **Data transformation**: Converting input data from one format to another
- **Computed properties**: Calculating values based on other component
  properties
- **API interactions**: Fetching data from external services to populate
  component properties
- **Complex data manipulation**: Using lodash or custom logic to transform data
  structures

## See Also

For detailed examples and technical implementation details, see the
[Attribute Function Examples](/reference/function#attribute-function-examples)
section in the Functions Reference.
