# Vocabulary

This is a reference to all of the vocabulary used in System Initiative.

## Actions

Actions are specific operations that are queued up to change your
infrastructure, such as creating, refreshing, updating or deleting a real-world
resource.

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

## Context Menu

The Context Menu contains all available Actions to a Component at that time,
both in the Grid and Map view. The Context Menu enables a user to edit, pin,
duplicate, erase, delete, create templates, add to/remove from view and run
actions on Components.

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

## Collaborator

A collaborator is a member of a workspace. A collaborator can makes changes to
the model or assets within a change set but cannot apply those changes without
approval. Learn more about the [different roles](/reference/authorization-roles)
a member can have in a workspace.

## Component

A component is the 'theoretical' half of a Model. It represents the
configuration values you _want_ something to have, while the Resource represents
its real world values.

## Credential

A credential is a type of component that stores secret data and has
authentication functions attached to it. They are used to provide access to
cloud providers, etc.

## Entity

This word encapsulates relevant objects that can be mutated, used, or are actors
in System Initiative. The Audit Trail dashboard makes use of this term to
standardize all relevant objects seen in Audit Logs.

### Entity Type

This is the type of an entity. A "Component" and a "Func" are entity types in
System Initiative.

### Entity name

This is the name of an entity. A "Component" could have the name "Server" and a
Func could have the name "Validate Container Image".

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

## Grid

The Grid is a a tile like visualization of your created Components. On the Grid
page you can also see pending Actions and your Function run history.

## Map

The Map is a visual representation of your created Components and their
relationships via prop to prop Subscriptions. You can also create, edit, delete
etc. from the Map.

## Model

### A Model

When we refer to "a model", we mean a single Component/Resource pair.

### The Model

When we refer to "the model", we're talking about all of the assets, functions,
components, resources, etc. that make up your hypergraph.

## Qualification

Qualifications are functions that are executed to ensure that the component can
be applied successfully to the workspace. An example is the qualification
function that checks if the Secret you have passed into your AWS Credential
component are valid.

## Resource

A resource is the data about the real-world thing represented by a component.

## Schema

Assets in System Initiative are defined by a Schema. Each version of the schema
is referred to as a Schema Variant.

## Secret

Secrets are encrypted data stored for a given type of credential. They are
defined by a credential component.

## Subscription

Subscriptions create a connection between properties of different components,
allowing them to share and synchronize data. For example, if one component has
an ImageId property and another has an Image property, you can subscribe the
Image property to the ImageId. This ensures that the Image property stays up to
date whenever the ImageId changes.

## View

Views are a uniquely named and organized collection of components built to
create representations of infrastructure configurations that are semantically
relevant to a given workflow or user.

## Workspace

A workspace named space where all modelling and real world representations are
stored which can be accessed by a group of users. Objects such as assets,
functions, components, edges, resources, secrets, views, and change sets are
found in a workspace.
