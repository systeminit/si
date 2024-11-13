# Authorization Roles Reference

Authoriziation roles are modelled as
[ReBAC](https://en.wikipedia.org/wiki/Relationship-based_access_control) and
added to a workspace through the
[AuthPortal](https://auth.systeminit.com/workspaces).

## Role Types

There are 3 role types available within a workspace:

- Owner
- Approver
- Collaborator

### Owner

The owner of the workspace is the person who initially created it. They are the
person that can invite or remove other people from the workspace as well as
being able to change the role of the people in a workspace. A workspace only has
a single owner associated with it.

The owner of a workspace is automatically an approver of that workspace. This
means that they have the power to approve and merge a change set.

### Approver

A person can be designated as an approver for a workspace. That person can
approve change set merge requests for the workspace. The approver role does have
the permission to add more people to the workspace or to change the role of
other in the workspace.

A workspace can have multiple approvers.

### Collaborator

A collaborator is a person who can author and model within a workspace but who
cannot apply a change set to HEAD. A collaborator needs to request approval
before merging a change set. The collaborator role does have the permission to
add more people to the workspace or to change the role of other in the
workspace.

A workspace can have multiple collaborators.
