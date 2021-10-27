/*
  Changes Summary

  Summarize changes insights
   - current open changeset count
   - current change count for selected changeset

  Opened changesets
   - total count of opened changesets

  New nodes (for selected changeset)
   - total count of new nodes

  Deleted nodes (for selected changeset)
   - total count of deleted nodes

  Modified nodes (for selected changeset)
   - total count of modified nodes

  Node edits (for selected changeset)
   - total count of node edits (attributes)

*/

export interface Changes {
  openedChangesetCount: number;
  currentChangeset: Changeset;
}

interface Changeset {
  id: string;
  newNodes: number;
  deletedNodes: number;
  modifiedNodes: number;
  nodeEdits: number;
}

const myChangeset: Changeset = {
  id: "myChangeSet",
  newNodes: 2,
  deletedNodes: 0,
  modifiedNodes: 1,
  nodeEdits: 7,
};

export const changesData: Changes = {
  openedChangesetCount: 3,
  currentChangeset: myChangeset,
};
