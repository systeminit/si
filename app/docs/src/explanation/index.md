# Explanation

These articles help you understand the "why" behind System Initiative. They
explore concepts, design decisions, and architecture to give you a deeper grasp
of how things work.

## Cloud Providers

Learn how System Initiative connects to cloud providers and handles authentication. You can start with the [overview of supported providers](./cloud-providers/index.md) or you can learn about them individually:

- [AWS (Amazon Web Services)](./cloud-providers/aws.md) - Connect using static credentials, SSO, or assume role
- [Azure (Microsoft Azure)](./cloud-providers/azure.md) - Authenticate with Azure using Service Principal
- [DigitalOcean](./cloud-providers/digital-ocean.md) - Connect using API tokens
- [Hetzner Cloud](./cloud-providers/hetzner.md) - Authenticate with API tokens

## Getting Started

- [Create Workspace API Tokens](./generate-a-workspace-api-token.md) - Generate and manage tokens for API authentication
- [Enable Slack Webhook](./enable-slack-webhook.md) - Set up Slack notifications for your workspace
- [IaC vs System Initiative](./iac-comparison.md) - How System Initiative compares to traditional Infrastructure as Code tools
- [Policy Layers](./policy-layers.md) - How you can define and enforce policy across three distinct layers for your infrastructure
- [Working on System Initiative](./working-on-si.md) - Learn how to contribute to the project and collaborate with the team

## Architecture

Explore the design decisions and architectural principles behind System Initiative. You can start with the [overview of the distributed architecture powering the platform](./architecture/index.md) or dive into the individual pieces:

- [AI Native Collaboration](./architecture/ai.md) - The role of AI throughout the platform
- [Change Control](./architecture/change-control.md) - The design behind safe, auditable infrastructure changes
- [Digital Twin](./architecture/digital-twin.md) - Maintaining a live model of your infrastructure state
- [Function Execution Framework](./architecture/functions.md) - Where and how schema functions run
- [Tenancy and Access Control](./architecture/tenancy.md) - How System Initiative handles multiple workspaces and permissions
- [The Data Model](./architecture/snapshot.md) - The graph-based approach to modeling infrastructure
- [The Distributed Execution Engine](./architecture/engine.md) - How functions execute across System Initiative's infrastructure
