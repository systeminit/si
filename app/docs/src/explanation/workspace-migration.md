---
outline:
  level: [2, 3, 4]
---

# Migrating a workspace from socket connections to property subscriptions

As part of the launch of our new user experience, we have deprecated the use of
sockets connectionsand replacing it with a new mechanism for creating
relationships between components: Property Subscriptions. This is a multi step
migration process that we began undertaking on Saturday August 2nd 2025. As the
functionality we're moving to is not a 1:1 replacement, any existing socket
connections that cannot be automatically mapped to prop subscriptions, will be
retained, and will continue to function as expected, however these socket
connections will no longer be visible, able to be created, or modified going
forward.

Each component that has an unmigratable connection will show a warning and the
specific property that cannot be migrated. For these components, please contact
us on [Discord](https://discord.com/invite/system-init) and we would love to
help recreate, reauthor, or reconfigure the components in question to take
advantage of the new functionality.
