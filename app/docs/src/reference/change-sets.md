# Change Sets

Change Sets are the change control and simulation mechanism within System Initiative. They allow you to safely propose changes to your model, preview those changes, and then apply them to the real world when you are ready. As change sets are applied to your [workspace](./workspaces.md), your open change sets are automatically updated to reflect those changes.

:::tip
A change set is like an auto-rebasing git branch that can never have conflicts. Unlike tracking state in Git, they are tailor made for the dynamic, fluid nature of how infrastructure changes, regardless of how those changes were made.
:::


## The HEAD Change Set

The HEAD Change Set represents the current state of your resources in the real world. The all your components and resources, along with any actions that are currently enqueued, are tracked on the HEAD Change Set.

:::info
Think of the HEAD Change Set as your view of everything that is currently happening with your workspace. If you want to know "what is happening right now?", looking at HEAD is the way to get that answer.
:::

## Tracking & Auditing Changes

When you create a Change Set, you get an identical clone of HEAD at that time. As you make changes to component attributes, enqueue actions, develop schemas, or write functions, the Change Set tracks your modifications as an operational transform relative to HEAD. It is also tracked in an audit log that records who made the change, when they made it, what changed, and any downstream side-effects that happened as a result (for example, changing an attribute on one component might cause attributes on other components to change based on subscriptions - these are also tracked.)

:::tip
The ability to track the *downstream effects* of a particular update is a critical difference between Change Sets and Git. By having the underyling data model understand not only the specific change you made, but the downstream effects, System Initiative provides a comprehensive view of a change sets impact. It also provides an audit log that can attest to the provenance of every granular change.
:::


## How changes are applied

Each change is an [Operational Transfomation](https://en.wikipedia.org/wiki/Operational_transformation) to our core data-model. When a Change Set is applied to HEAD, the set of Operational Transforms are applied to the underlying [graph data model](../reference/architecture/snapshot.md). Each change is idempotent, convergent, and ordered in time - which ensures that as multiple changes are applied they will never conflict.

## Creating a Change Set

<DocTabs tabs="CLI,AI Agent,Web Application,Public API">
<TabPanel value="CLI">

To create a Change Set with the CLI:

```shellscript [Create a Change Set]
$ si change-set create deploy-production
```

</TabPanel>
<TabPanel value="AI Agent">

To create a Change Set with the AI Agent:

```prompt
> Create a deploy-production change set
● Successfully created the deploy-production change set!

  Details:
  - Change Set ID: 01KBR9QXRX1P3X0VJ8FCMEJ6QS
  - Status: Open
  - Ready for modifications

  You can now create, modify, or import components in this
  change set. When you're ready to apply the changes to
  the real world, you can apply the change set to merge it
  into HEAD.
```

</TabPanel>
<TabPanel value="Web Application">

To create a Change Set:

![Creating a Secret](./change-sets/create-a-change-set.png)

1. Press the `C` hotkey or click the 'Create Change Set' button from the top bar.
2. Enter the Change set name
3. Click the 'Create change set' button

</TabPanel>
<TabPanel value="Public API">

:::code-group
```typescript [TypeScript]
const response = await changeSetsApi.createChangeSet({
  workspaceId,
  createChangeSetV1Request: {
    changeSetName: 'my-change-set',
  },
});
```

```python [Python]
request = CreateChangeSetV1Request(
    change_set_name="my-change-set",
)

response = change_sets_api.create_change_set(
    workspace_id=workspace_id,
    create_change_set_v1_request=request,
)
```
:::

Both examples use the [Public Create Change Set API](./public-api#create-a-change-set). See the [Public API](./public-api) documentation and the SDKs for more details on configuring the SDK to communicate with System Initiative.


</TabPanel>
</DocTabs>

