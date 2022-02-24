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
