# Explanation

These articles help you understand the "why" behind System Initiative. They
explore concepts, design decisions, and architecture to give you a deeper grasp
of how things work.

## Cloud Providers

Learn how System Initiative connects to cloud providers and handles
authentication:

- [Cloud Providers Overview](./cloud-providers/index.md) - An overview of
  supported cloud providers and how System Initiative works with them
- [AWS (Amazon Web Services)](./cloud-providers/aws.md) - Connect using static
  credentials, SSO, or assume role
- [Azure (Microsoft Azure)](./cloud-providers/azure.md) - Authenticate with
  Azure using Service Principal
- [DigitalOcean](./cloud-providers/digital-ocean.md) - Connect using API tokens
- [Hetzner Cloud](./cloud-providers/hetzner.md) - Authenticate with API tokens

## Getting Started

- [Working on System Initiative](./working-on-si.md) - Learn how to contribute
  to the project and collaborate with the team
- [Enable Slack Webhook](./enable-slack-webhook.md) - Set up Slack notifications
  for your workspace
- [Create Workspace API Tokens](./generate-a-workspace-api-token.md) - Generate
  and manage tokens for API authentication
- [IaC vs System Initiative](./iac-comparison.md) - How System Initiative
  compares to traditional Infrastructure as Code tools

## Architecture

Explore the design decisions and architectural principles behind System
Initiative:

- [System Initiative Architecture](./architecture/index.md) - A look at the
  distributed architecture powering the platform
- [The Distributed Execution Engine](./architecture/engine.md) - How functions
  execute across System Initiative's infrastructure
- [The Data Model](./architecture/snapshot.md) - The graph-based approach to
  modeling infrastructure
- [Change Control](./architecture/change-control.md) - The design behind safe,
  auditable infrastructure changes
- [Digital Twin](./architecture/digital-twin.md) - Maintaining a live model of
  your infrastructure state
- [Function Execution Framework](./architecture/functions.md) - Where and how
  schema functions run
- [Tenancy and Access Control](./architecture/tenancy.md) - How System
  Initiative handles multiple workspaces and permissions
- [AI Native Collaboration](./architecture/ai.md) - The role of AI throughout
  the platform
