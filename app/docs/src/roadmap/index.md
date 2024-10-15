---
outline:
  level: [2, 3, 4]
---

# Road map

Updated 2024-09.

Want to influence the road map? [Learn more about how to work on System Initiative](../explanation/working-on-si.md).

## Growing coverage of cloud platforms

We've been primarily focused on AWS, and there is plenty more to model there.
We hope to start on GCP, Azure, and other platforms as well. If there is
something specific you want or need, come chat with us on
[Discord](https://discord.com/invite/q6H2ZyFh) anytime, but specifically
on Authoring Friday's!

## Management components

Think of this like templates and workflows smashed together. Essentially
components that can create and manage other components. Think "applications"
that take properties, and then expand into the required infrastructure; or have
deploy actions that pull from artifact repositories and then run actions across
the infrastructure.

### Current Opportunity: Management Functions

<iframe width="560" height="315" src="https://www.youtube.com/embed/GKOtMulPTMc?si=o7GVGMXeKcr37-g_" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

Management functions allow a model to manage its own attributes; create,
update, and delete components; and enqueue actions. In their first iteration,
they’ll be used for three big use cases: importing existing cloud resources,
modular templating, and management of existing components. [Read the blog post to learn more.](https://www.systeminit.com/blog/opportunity-management-functions)

## Discovery of existing cloud resources

This starts with having an Import feature, that will let you connect any
component to an existing resource. Eventually, we would like this to expand
to also discovering related infrastructure.

## More ways to visualize

You can collapse and expand infrastructure today, but we want to be able to
create custom views, allow you to drill down, etc.

## Enterprise features

Custom authentication, ubiquitous access control, history, etc.

### Current Opportunity: ReBAC

<iframe width="560" height="315" src="https://www.youtube.com/embed/QlWaeJH74Bo?si=uBXbQ5kyeynFSzjQ" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

We are laying the foundation of a relationship based access control (ReBAC)
system using SpiceDB. This first iteration of work will allow a workspace to
have a defined set of approvers who curate the changes to the infrastructure
before it is applied (and therefore, before making any changes to real
infrastructure).
[Read the blog post to learn more.](https://www.systeminit.com/blog/opportunity-rebac)

## Scaling to huge infrastructures

This is both visual scale, but also front and back-end scale.
