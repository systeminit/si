---
outline:
  level: [2, 3, 4]
---

# Road map

Updated December 13, 2024.

Want to influence the road map?
[Learn more about how to work on System Initiative](../explanation/working-on-si.md).

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

### Import (Generally Available)

This will let you connect any component to an existing resource.

Read our
[announcement blog post](https://www.systeminit.com/blog/announcing-resource-import)
to learn more about import functionality

### Discovery of existing cloud resources

Discover builds on import where you can discover all infrastructure within a
cloud environment

### Visual Templates (In development)

Visual templates will allow you to model your infrastructure and then extract
that to a management function. The management function will keep all the
geometry and connections between the components as well as the properties of the
components. The management function will get added to a newly created asset
which you can reuse on the diagram.

## Enterprise Features

Custom authentication, ubiquitous access control, history, etc.

### Approval Workflows via ReBAC (Generally Available)

<iframe width="560" height="315" src="https://www.youtube.com/embed/QlWaeJH74Bo?si=uBXbQ5kyeynFSzjQ" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

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

### Fine Grained Access Control (In planning)

We are providing the ability to attach approvers to specific sets of components,
via views. This means core parts of the infrastructure can have approvers
attached to them ensuring safer changes to the infrastructure.

## Views (Generally Available)

<iframe width="560" height="315" src="https://www.youtube.com/embed/qpNxaAojuzI?si=XLnnJy7uWF4ruEVY" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

Views allow the creation of custom views within the workspace. Views are a
semantic representation of a group of components and components can exist in
multiple views. Components in a view can connect to components in another view.
This enables teams to create diagrams that are specific to their application or
area of specialization.
[Read the blog post to learn more.](https://www.systeminit.com/blog/opportunity-views)

## GitHub Actions to drive SI (In development)

We are building the ability for a user to embed System Initiative in their CI/CD
pipeline. We are building the capability to create an access token for a
workspace, pass data to a component and trigger a management function and check
the status of the resulting action.

## Growing coverage of cloud platforms

We are using
[Generative AI](https://en.wikipedia.org/wiki/Generative_artificial_intelligence)
as a way to accelerate the speed at which we can provide high-quality assets. We
are still primarily focused on AWS, as we need to increase the coverage and
quality of the assets there, but we hope to start on Google Cloud, Azure and
other platforms as well. If there is something specific you want or need, come
chat with us on [Discord](https://discord.com/invite/q6H2ZyFh) anytime.

This is currently in internal testing behind a feature flag and is anticipated
to be Generally Available, January 2025.
