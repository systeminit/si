interface ChangeSetConstructor {
  name: string;
  workspaceId: string;
  createdByUserId: string;
}

class ChangeSet {
  name: string;

  constructor(args: ChangeSetConstructor) {}
}

class ChangeSetGenerator {}
