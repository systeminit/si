---
outline: [2, 3]
---

# Audit Logs

Audit Logs provide real-time visibility into all changes and events within your workspace. Through a structured, spreadsheet-like interface, you can see who made what changes, when they made them, and what the downstream effects were across your infrastructure model.

:::info
Audit Logs are currently only accessible through the web application interface. They are available on a per-change-set basis, allowing you to audit changes in your working change set or review the complete history in HEAD.
:::

## Accessing Audit Logs

Audit Logs are accessed through the web application by navigating to the Audit view within your workspace. The logs are scoped to the currently selected [Change Set](./change-sets.md), showing all events that have occurred within that change set.

When viewing HEAD, you'll see the complete audit history of all changes that have been applied to your workspace. When viewing a specific change set, you'll only see the changes made within that change set.

## Understanding Audit Log Entries

Each audit log entry captures a specific event within System Initiative and contains:

- **Event**: The type of action that occurred (e.g., "Deleted", "Updated Component", "Created")
- **Entity Type**: The kind of object affected (e.g., Component, Property, Schema, Function)
- **Entity Name**: The specific name of the object that was changed
- **Change Set**: The change set where the event occurred
- **User**: Who performed the action
- **Time**: When the event occurred

### Event Types

Audit Logs track a wide range of events, including:

- **Component Events**: Creating, updating, deleting, and restoring components
- **Property Changes**: Updates to component attributes, including before and after values
- **Schema Operations**: Installing, upgrading, or modifying schemas
- **Function Changes**: Creating, updating, or deleting functions
- **Action Events**: Enqueueing, executing, or canceling actions
- **Connection Changes**: Adding or removing connections between components
- **Change Set Operations**: Creating, applying, or abandoning change sets

## Viewing Detailed Information

Each audit log entry can be expanded to reveal detailed metadata in JSON format. This metadata is specific to the event type and provides comprehensive information about what changed.

For example:
- **Property updates** show both the before and after values
- **Component deletions** include the component ID and schema variant information
- **Action events** contain execution details and results

To expand an entry, click on the row in the audit log table. The detailed JSON will be displayed below the entry.

## Filtering Audit Logs

The Audit Log interface provides powerful filtering capabilities to help you find specific events:

- **By Event Type**: Filter to show only creates, updates, deletes, or other specific event types
- **By Entity Type**: Focus on changes to components, properties, schemas, functions, or other entity types
- **By Entity Name**: See all events related to a specific component or entity
- **By Change Set**: Filter events by which change set they occurred in
- **By User**: View changes made by specific team members

Multiple filters can be applied simultaneously to narrow down your search. Filters can be cleared individually or all at once.

## Loading More Entries

The Audit Log initially loads the most recent entries. To view older events, click the "Load 50 More" button at the bottom of the log. You can continue loading additional entries until all available logs have been displayed.

## Understanding Change Tracking

Audit Logs are deeply integrated with System Initiative's [Change Sets](./change-sets.md) functionality. Each change you make in a change set is recorded with:

1. **Direct Changes**: The specific modification you made (e.g., updating a component attribute)
2. **Downstream Effects**: Automatic changes that occurred as a result (e.g., subscribed components that updated based on your change)
3. **Provenance Information**: Complete context about who, what, when, and where

:::tip
The ability to track downstream effects is a critical feature of Audit Logs. When you change one component, subscriptions may cause changes to cascade to other components. The Audit Log captures all of these effects, giving you complete visibility into the impact of your changes.
:::

## Use Cases

Audit Logs support several important workflows:

### Troubleshooting
When something unexpected happens, use Audit Logs to trace back through recent changes and identify what triggered the issue.

### Compliance and Governance
Audit Logs provide an immutable record of who made what changes and when, supporting compliance requirements and audit trails for regulated environments.

### Understanding Impact
Before applying a change set, review the Audit Log to see all the changes and their downstream effects, ensuring you understand the full scope of what will be applied.

### Collaboration
In team environments, use Audit Logs to see what your teammates are working on and understand the history of changes to the workspace.

### Learning and Documentation
New team members can review Audit Logs to understand how infrastructure has evolved and learn from the changes others have made.

## Related Concepts

- [Change Sets](./change-sets.md) - The change control mechanism that Audit Logs track
- [Components](./components.md) - Component changes are recorded in Audit Logs
- [Actions](./actions.md) - Action lifecycle events appear in Audit Logs
- [Functions](./function.md) - Function changes are tracked in Audit Logs

## Technical Details

For developers interested in the structure and format of Audit Log data, see the [Audit Trail Reference](./audit-trail.md), which provides detailed JSON schemas and metadata specifications for each event type.
