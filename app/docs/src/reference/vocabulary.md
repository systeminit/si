# Vocabulary

This a reference to all of the vocabulary used in System Initiative.

## Asset

Anything that can be used on the canvas.

## Credential

A credential is a type of component that stores secret data, and has
authentication functions attached to it. They are used to provide access to
cloud providers, etc.

## Change Set

Change Sets represent a batch of changes to the components, assets, functions
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
[getting started tutorial](/tutorial/getting-started).

### Applying a Change Set

When you _apply_ a change set, all of the properties and code are merged to
HEAD, and any actions enqueued. Learn how to apply a change set in the
[getting started tutorial](/tutorial/getting-started).

### Abandoning a Change Set

Abandoning a change set essentially deletes it, and all of its proposed changes,
from the system.

## Hypergraph of functions

The hypergraph of functions, or hypergraph, is the data structure that powers
System Initiative. It is a graph that represents all of the code, components,
resources, and functions that are in a workspace. We call it a "hypergraph"
because it is multi-dimensional through the use of change sets.

## A Model

When we refer to "a model", we mean a single Component/Resource pair.

## The Model

When we refer to "the model", we're talking about all of the assets, functions,
components, resources, etc. that make up your hypergraph.

## Resource

A resource is the data about the real-world thing represented by a component.

## Secret

Secrets are encrypted data stored for a given type of credential. They are
defined by a credential component.
