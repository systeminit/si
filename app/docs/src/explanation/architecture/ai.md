---
outline: false
prev:
  text: 'Tenancy and Access Control'
  link: '/explanation/architecture/tenancy'
next:
  text: 'Overview'
  link: '/explanation/architecture/index'
---

# AI Native Collaboration

System Initiative exposes its engine to external AI agents through the Model Context Protocol (MCP).  This allows agents to perform the same operations as human users:

- Query snapshot state and component configurations
- Create and manage change sets
- Execute functions within their permission context
- Propose and apply infrastructure modifications
- Participate in real-time collaborative editing sessions

In practice, this makes AI agents first-class collaborators.  They can co-edit infrastructure alongside humans, with changes propagated in real time via the same operational transform mechanism.  Concurrency is controlled at the engine level, ensuring that divergent perspectives are synchronized through replay. Change sets provide an isolated environment for AI agents to safely mutate the system, while humans-in-the-loop can review proposed changes and apply the change set when they’re ready.
