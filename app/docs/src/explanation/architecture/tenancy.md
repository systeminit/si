---
outline:
  level: [2, 3, 4]
prev:
  text: 'Function Execution Framework'
  link: '/explanation/architecture/functions'
next:
  text: 'AI-Native Collaboration'
  link: '/explanation/architecture/ai'
---

# Tenancy and Access Control

## Workspace Isolation

Workspaces provide complete tenant isolation through:

- Data Separation: Each workspace maintains its own snapshot and change set collection
- Secret Isolation: Credentials are encrypted with workspace-specific keys and cannot cross the workspace boundary
- Access Isolation: Users, agents, and integration access are scoped to a workspace, and access to additional workspaces must be explicitly granted

## Unified Access Control Model

Both human users and AI agents operate under the same permission system:

- Role-Based Permissions: Static permissions assigned through workspace roles (approver, collaborator)
- Relationship-Based Permissions: Dynamic permissions can be applied to views, such that any change to a component within the given view can trigger additional approvals for discrete sets of users. This enables the designation of resource owners, ensuring the right people can approve changes.

## AI Agent Integration

AI agents authenticate using [Personal Access Tokens](../../explanation/generate-a-workspace-api-token.md) that inherit identical permission constraints. The permission system prevents privilege escalation while enabling AI agents to perform complex infrastructure operations within their authorized scope.
