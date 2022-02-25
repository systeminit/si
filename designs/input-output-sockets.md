# Data flow between Sockets, Components, and Props

## Requirements

* No data cycles allowed (`Prop A -> Prop B -> Prop A` and `Prop A
  (Component A) -> Prop B (Component B) -> Prop A (Component A)` are
  forbidden).
* Able to determine which functions need to be re-run (across all
  Components), whenever a Prop's value changes.
* Functions do not need to know full, internal structure of other
  Components.

## Architecture

* All data between Components flows through the Sockets of the Components.
* Components have zero or more Input Sockets.
* Components have zero or more Output Sockets.
* An Output Socket consists of:
  * A source Prop
  * A transformation function
  * An output type
* An Input Socket consists of:
  * An input type

### Output Socket

#### Source Prop

* An Output Socket for a Component has a source Prop that is the input
  to the Output Socket's function.

Given a Component that looks like:

```json
{
  "prop_one": "some value",
  "prop_two": [
    { "nested_prop": "foo" },
    { "nested_prop": "bar" },
  ],
}
```

If we declare an Output Socket, and say that its input is
`"prop_two"`, it would see the following as its input:

```json
[
  { "nested_prop": "foo" },
  { "nested_prop": "bar" },
]
```

And if the Output Socket is bound to `"prop_one"`, it would get
`"some_value"` as its input.

An Output Socket is bound to a Prop, instead of to an
AttributeResolver, since we need a stable identifier for where we
should start gathering the data to feed into the Output Socket's
function. This means that it probably does not make sense to allow
binding to a Prop that is nested inside of an Array, or Map, and that
the Output Socket should likely be bound to the outer Array or Map
directly.

#### Transformation Function

The Transformation Function receives as its input a subset of a
`ComponentView` that starts at the Prop that the Output Socket has
been bound to. The simplest Transformation Function is the identity
function that returns exactly the data as it was given. The
Transformation Function is allowed to do anything that it likes to the
partial `ComponentView` that it is provided, as long as the shape of
the data that it returns conforms to the Output Socket's Output Type
definition.

### Output Type & Input Type

All Output and Input Sockets have a type that is expressable as a
TypeScript Interface (though this is not nececessarily the internal
representation). This type definition is used to match an Output Type
against the Input Type definition on Input Sockets to determine which
Input Sockets can receive data from a particular Output Socket.

#### Identical Input & Output Types

An initial implemntation would check that the Output and Input Sockets
have identical type definitions.

Output Socket type definition:

```typescript
interface Container {
  imageName: String,
  ports: {
    portStart: Number,
    portEnd: Number,
    protocol: String,
  }[],
}
```

Input Socket type definition:

```typescript
interface Image {
  imageName: String,
  ports: {
    portStart: Number,
    portEnd: Number,
    protocol: String,
  }[],
}
```

#### Output Type as a superset of Input Type

This could later be extended to allow matching an Output Socket to an
Input Socket where the Output Socket's type is a strict superset of
the Input Socket's type. For example, the following Output Socket type
would be compatible with the following Input Socket's type.

Output Socket type definition:

```typescript
interface Container {
  imageName: String,
  ports: {
    portStart: Number,
    portEnd: Number,
    protocol: String,
  }[],
}
```

Input Socket type definition:

```typescript
interface Image {
  imageName: String
}
```

#### Infallable conversions between Output Type & Input Type

A further extension of the type compatibility between Output and Input
Sockets could be to allow for an Input Socket to allow for type
conversions that cannot fail. This would allow for things like
accepting an Output Socket that is providing a `Number` in a place
where the Input Socket is expecting a `String`.

#### Input Type and SocketArity

If the Input Socket is defined as having `SocketArity::One`, then the
type definition is used as-is, and only one Output Socket can be
connected to it.

If the Input Socket is defined as having `SocketArity::Many`, then the
type definition is considered to be the definition for a single
element in an array, and zero or more Output Sockets can be connected
to it.

If an Output Socket's type is the same as that of the Input Socket,
then the single element emitted by the Output Socket is pushed onto
the array representation of the Input Socket.

If an Output Socket's type is an array whose element type matches the
type of the Input Socket, then the elements of the array emitted by
the Output Socket are pushed onto the array representation of the
Input Socket.

In the case that multiple Output Sockets are connected to the Input
Socket, the ordering of array elements in the Input Socket **is not
guaranteed**.

### Prop func bindings

Functions for a Prop declare their usage of any Input Sockets in the
same Schema Variant, and any usage of other Props from the same Schema
Variant. Any declared usage of Input Sockets, or Props that results in
a cycle is an error, and should be reported as such. All cycles are
disallowed, regardless of whether they are across multiple Components,
or within a single Component.

### Triggering re-execution of functions

Whenever a value for a property is updated, a dependency graph is
built with the updated property as the root of the tree using the
information from:
* The declared usage of other properties by a function
* The connections between Props and Output Sockets. This means that if
  an Output Socket is bound to the parent Prop (of any depth) of a
  Prop that has had its value change, that Output Socket is considered
  to have changed, and must be added to the dependency graph.
* The connections between Output Sockets and Input Sockets, and the
  usage of Input Sockets by functions. For any Output Socket that is
  considered to have changed, all functions that declare that they use
  an Input Socket that is connected to that Output Socket must be
  considered to have changed, and added to the dependency graph.

Once this graph has been built, it must be executed in topological
order to ensure that functions "further along" in the graph are
operating on the updated information from previous function
executions.

While the above describes how to build the **full** graph, doing so
all at once is not necessary, and can be done limiting scope to the
"nearest neighbors" of the Prop that has changed, reducing the amount
of state that must be kept to keep track of what executions need to be
done.

## Implementation Plan

This plan's layout is reminiscent of an iteration story map, but not concerned with ordering.
Due to its unordered nature, readers may need to skim through all sections before fully internalizing each objective.

In addition, each section contains three estimates for time to completion for each object.
The total of these estimates calculates to the following (note: "days" are "business days"):

- Best Case: 14 days
- Worst Case: 40 days
- Likely Case: 21 days

### Re-define or clearly define `Attribute`

We have a `Component` and we give a value to the corresponding `Prop` for that `Component`.
That value is an `Attribute`.
`Attribute` will become the instance of a `Prop`.

- Best Case: n/a
- Worst Case: n/a
- Likely Case: n/a

### `Funcs` bound to `Props` need to be able to specify what their inputs are

**Possible implementations:** this ability could be achieved via binding a `HashMap<String, InputSocketId>` to a field in `Prop`.
With this, we can build up a hash of key `String` (what we should call the value of that input socket ID when we pass it into the func) and resolution of what value that `InputSocketId` corresponds with.
This must be manually specified by us or function users.

- Best Case: 1 day
- Worst Case: 3 days
- Likely Case: 1 day

### When running a `Func`, we need to build a hash of `InputSocket` values and pass then into the `Func`

A `Func` expects a hash as its argument.
This section focuses on the "actual assembly" of the hash.

- Best Case: 1 day
- Worst Case: 5 day
- Likely Case: 3 day

### Split `AttributeResolver`'s duties of determining what function should be run and what the result of running a function was

The `Attribute` prefix on the name might not be accurate since it extends into sockets, beyond attributes (might be a `Socket`, `Prop`, or `Attribute`).
The goal is to retire `AttributeResolver` and create two replacements: resolving functions and resolving values.
The prototype would occupy the former duty and the latter would occupy the latter duty.

This is one of the larger timesinks as we would not only need to refactor the existing usage of `AttributeResolver`, but we would also need to ensure what each `AttributeResolver`'s duties are: function resolution, value resolution, or both.

- Best Case: 3 days
- Worst Case: 10 days
- Likely Case: 5 days

### Create an `InputSocket`

This object needs... 

- a way to define the shape of the type that it accepts
- a way to know what `SchemaVariant` this object is attached to
- a way to know whether or this is implicit (perhaps, declaratively)
- a field containing `AttributeResolverContext`
  - specifically, for dynamic determination of which `FuncBindingReturnValue` to use depending on what context the component is in
  - at minimum, this context needs to be one used for lookups (needs `Prop`, and `SchemaVariant`, but not `Component`, `System`, or `Application`)

There needs to be an implicitly created `InputSocket` for every `Prop` in a `SchemaVariant`.
This should be automatic and the `InputSocket` should be marked as internal only.
This is useful for intra-`Component` functions (e.g. setting "name" results in "image" being set to the same string for Docker Image).

We use the attached resolver context and attach the specificity we need (`Application`, `Component`, `System`) when we need to find the value of the `InputSocket`.

This object will have its own table.

- Best Case: 2 days
- Worst Case: 4 days
- Likely Case: 3 days 

### Create an `OutputSocket`

This object needs...

- the ability to bind a `Func` to it
- to know which `Prop` it belongs to (it can only ever belong to a `Prop`)
- to define the type that it emits (e.g. JSON Schema)

This object will have its own table.

- Best Case: 2 days
- Worst Case: 4 days
- Likely Case: 2 days

### Extend (and potentially rename) `AttributeResolverContext` to create a way to know which sockets are connected

This object needs an `OutputSocketId` that is of equal precedence to `PropId` and/or `InputSocketId`.
These are conceptually, mutually exclusive because if the `PropId` is specified for an `AttributeResolverContext` on an `InputSocket`, then that means it is for a `Prop`-to-`Prop` data flow within a `Component`.
If the `OutputSocketId` is specified, that means that the `InputSocket` is for cross-`Component` data flow.
This means that we should not have both IDs set on the same resolver context. 

```rust
pub struct AttributeResolverContext {
    // Cannot set this and OutputSocketId
    prop_id: PropId,
    component_id: ComponentId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    system_id: SystemId,
    // New field, cannot set this and PropId
    ouput_socket_id: OutputSocketId
}
```

- Best Case: 1 day
- Worst Case: 3 days
- Likely Case: 2 days

### Create root `Prop` of kind `PropObject` for all `SchemaVariant`s

This root `Prop` will have two child `Prop`s (both are also of kind `PropObject`):

1. SI-based attributes (e.g. "name")
2. Domain-based attributes (i.e. model representing the domain concept)

Now, our behavior is unified between `Attribute`s that are, *and are not*, `Prop`s.

**Potential timesink (1/2):** dealing with special logic with attributes, like "name".
However, "name" would be handled like any other `Prop` in this case, so it might be more ergonomic.
In our implementation, everything is a `Prop`, so our consideration would have to factor domain-model `Props` versus SI-based `Props`.
In the current codebase, this is primarily found on the `update_from_edit_field` methods for multiple objects.

**Potential timesink (2/2):** implications for working with components.

- Best Case: 3 days
- Worst Case: 8 days
- Likely Case: 4 days

### Determine type resolution strategy for sockets

Early findings point towards using [jsonschema](https://github.com/Stranger6667/jsonschema-rs).

- Best Case: 1 day
- Worst Case: 3 days
- Likely Case: 1 day

### (Bonues) Potential Timesinks

Item | Potential Timesink?
--- | ---
Tenancy | No
Visiblity | No
Veritech and Cyclone | Yes, but only in the function payload shapes (if at all?)
SDF and the Frontend App | Yes, but they are descoped and affected routes and behavior will be disabled
Assumptions based on attributes that are not currently props | Yes
Schema and schema variant builtins migration | Yes, but only with concern to the _depth_ of props changed since schemavariants will now have a root `Prop` of kind `PropObject`
Qualifications | Yes, `ComponentView` from `cyclone` might change its shape and/or behavior to match the new root `Prop` of kind `PropObject` shape
Validations | No, since their comparison is internal to the `Prop`
Resource | Likely no since we have to ensure that resource sync on the component still works after refactor
Frontend Sockets, NodeView, and Schematic | Yes, but have descoped fixing the SDF and frontend focused portions from this plan (they may become disabled in the interim)
