# Vocabulary

This is a reference to all of the vocabulary used in System Initiative.

## Asset

Anything that can be used on the canvas.

## Arity

The number of arguments taken by a function. Most frequently used to specify how
many edges can connect to a socket (either "one" or "many".)

## Change Set

Change Sets represent a batch of changes to the components, assets, functions,
and actions in a workspace. When you want to propose a change in the real world,
you first create a change set to do it. Nothing you do in a change set should
alter the real world.

If you are familiar with version control systems like git, you can think of a
change set as an automatically rebasing branch.

When a change set is merged to head, any other open change sets in the workspace
are automatically _rebased_ against HEAD - meaning the data within them will be
updated to reflect the proposed changes to the model on HEAD.

### Creating a Change Set

Creating a change set makes a new copy of the hypergraph based on the current
data in HEAD. Learn how to create a change set in the
[Getting Started tutorial](/tutorials/getting-started).

### Applying a Change Set

When you _apply_ a change set, all of the properties and code are merged to
HEAD, and any actions enqueued. Learn how to apply a change set in the
[Getting Started tutorial](/tutorials/getting-started).

### Abandoning a Change Set

Abandoning a change set essentially deletes it, and all of its proposed changes,
from the system.

## Component

A component is the 'theoretical' half of a Model. It represents the
configuration values you _want_ something to have, while the Resource represents
its real world values.

## Credential

A credential is a type of component that stores secret data and has
authentication functions attached to it. They are used to provide access to
cloud providers, etc.

## Configuration Frame (down)

A "down" frame is one where the output sockets of the frame will automatically
connect to any components placed within the frame.

## Configuration Frame (up)

An "up" frame is one where the output sockets of the components within the frame
will connect to the input sockets of the frame.

## HEAD

HEAD is the change set that represents "the real world". It can only be altered
by actions, refreshing resources, or applying change sets.

## Hypergraph of functions

The hypergraph of functions, or hypergraph, is the data structure that powers
System Initiative. It is a graph that represents all of the code, components,
resources, and functions that are in a workspace. We call it a "hypergraph"
because it is multi-dimensional through the use of change sets.

We know it's not an actual
[hypergraph](https://en.wikipedia.org/wiki/Hypergraph). It's just less of a
mouthful than 'multi-dimensional graph'.

## Input Socket

A socket that feeds data into a component. Appears on the left side of
components.

## Model

### A Model

When we refer to "a model", we mean a single Component/Resource pair.

### The Model

When we refer to "the model", we're talking about all of the assets, functions,
components, resources, etc. that make up your hypergraph.

## Output Socket

A socket that feeds data out of a component. Appears on the right side of
components.

## Qualification

Qualifications are functions that are executed to ensure that the component can
be applied successfully to the workspace. An example is the qualification
function that checks if the Secret you have passed into your AWS Credential
component are valid.

## Resource

A resource is the data about the real-world thing represented by a component.

## Secret

Secrets are encrypted data stored for a given type of credential. They are
defined by a credential component.
