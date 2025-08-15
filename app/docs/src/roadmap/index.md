---
outline:
  level: [2, 3, 4]
---

# Road map

Updated August 16, 2025.

Want to influence the road map?
[Learn more about how to work on System Initiative](../explanation/working-on-si.md).

## Job specific interface (In public beta)

We are adding a more purpose-built job specific user interface for interacting
with System Initiative. The user interface will be focused on the model itself,
allowing you to get to the work - building, editing, and visualizing
infrastructure - quicker.

## System Initiative MCP Server (In public beta)

We are introducing a dedicated MCP server for System Initiative. This server
enables you to bring AI directly into your infrastructure workflows—acting as a
companion that understands your context, helps you generate and refine
resources, and accelerates your path from idea to working infrastructure.

## Better data connectivity (In public beta)

We are enabling a user to be able to connect any data within the system.
Currently, data connections are made via sockets, but we want to allow deeper
connectivity of data between infrastructure

## External API for integrations to System Initiative (In public beta)

We are building an API that can be used from 3rd party applications (like CI/CD
systems or custom internal developer platforms) to manage your infrastructure
using System Initiative.

## Management components

Think of this like templates, import and workflows smashed together. Essentially
components that can create and manage other components. Think "applications"
that take properties, and then expand into the required infrastructure; or have
deploy actions that pull from artifact repositories and then run actions across
the infrastructure.

<iframe width="560" height="315" src="https://www.youtube.com/embed/GKOtMulPTMc?si=o7GVGMXeKcr37-g_" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

[Read the blog post to learn more.](https://www.systeminit.com/blog/opportunity-management-functions)

### Management Functions (Generally Available)

Management functions allow a model to manage its own attributes; create, update,
and delete components; and enqueue actions. In their first iteration, they’ll be
used for three big use cases: importing existing cloud resources, modular
templating, and management of existing components.

Read our
[announcement blog post](https://www.systeminit.com/blog/announcing-management-functions)
to learn more about management functions.

### Import (Generally Available)

This will let you connect any component to an existing resource.

Read our
[announcement blog post](https://www.systeminit.com/blog/announcing-resource-import)
to learn more about import functionality

### Discovery of existing cloud resources

Discover builds on import where you can discover all infrastructure within a
cloud environment

### Visual Templates (Generally Available)

<iframe width="560" height="315" src="https://www.youtube.com/embed/HSqf1w-PJco?si=hHMfnNx9VzEuFIf6" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

Visual templates will allow you to model your infrastructure and then extract
that to a management function. The management function will keep all the
geometry and connections between the components as well as the properties of the
components. The management function will get added to a newly created asset
which you can reuse on the diagram.

[Read the blog post to learn more.](https://www.systeminit.com/blog/opportunity-visual-templates)

## Enterprise Features

Custom authentication, ubiquitous access control, history, etc.

### Approval Workflows via ReBAC (Generally Available)

We are laying the foundation of a relationship based access control (ReBAC)
system using SpiceDB. This first iteration of work allows a workspace to have a
defined set of approvers who curate the changes to the infrastructure before it
is applied (and therefore, before making any changes to real infrastructure).

Read our
[announcement blog post](https://www.systeminit.com/blog/announcing-approval-workflows)
to learn more about our approval workflows

### Audit History (Generally Available)

We are providing the functionality that allows users of a workspace to see every
change that has happened, who made them and when. The first iteration of this
work will allow users to be able to have basic sorting and filtering of events.

Read our
[announcement blog post](https://www.systeminit.com/blog/announcing-audit-trail)
to learn more about audit history.

### Fine-Grained Access Control (Generally Available)

We are providing the ability to attach approvers to specific sets of components,
via views. This means core parts of the infrastructure can have approvers
attached to them ensuring safer changes to the infrastructure.

Read our
[announcement blog post](https://www.systeminit.com/blog/announcing-fine-grained-access-control)
to learn more about fine-grained access control.

## Views (Generally Available)

Views allow the creation of custom views within the workspace. Views are a
semantic representation of a group of components and components can exist in
multiple views. Components in a view can connect to components in another view.
This enables teams to create diagrams that are specific to their application or
area of specialization.

Read our
[announcement blog post](https://www.systeminit.com/blog/announcing-views) to
learn more about views.

## GitHub Actions to drive SI (Generally Available)

<iframe width="560" height="315" src="https://www.youtube.com/embed/UCZzTjBWSpg?si=co2wa47XcG4sqwfu" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

We are building the ability for a user to embed System Initiative in their CI/CD
pipeline. We are building the capability to create an access token for a
workspace, pass data to a component and trigger a management function and check
the status of the resulting action.

[Read the blog post to learn more.](https://www.systeminit.com/blog/opportunity-github-actions-to-trigger-system-initiative)

## Editor Integration (Generally Available)

We are building a FUSE-based file system to allow for a user to mount a
workspace and get access to all of the schemas and functions within their
workspace. This means a user can use their local IDE to be able to author in
System Initiative.

[Read the blog post to learn more.](https://www.systeminit.com/blog/opportunity-editor-integration-with-system-initiative)

## Full Coverage of the AWS Cloud Control API (Generally Available)

We have an integration with the
[AWS Cloud Control API](https://aws.amazon.com/cloudcontrolapi/), providing
complete coverage of all AWS resources supported by
[CloudFormation](https://aws.amazon.com/cloudformation/). This enables our
platform to create, read, update, and delete any AWS resource CloudFormation
supports.

This integration eliminates manual resource modeling and automatically
synchronizes with AWS's service catalog. As AWS releases new resources or
updates existing ones through CloudFormation, our platform supports these
changes without additional development work.

If there is something specific you want or need from these assets, or if you
need support for something outside of AWS, come chat with us on
[Discord](https://discord.com/invite/q6H2ZyFh) anytime.
