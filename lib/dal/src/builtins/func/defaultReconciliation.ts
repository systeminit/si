function reconciliation(arg: Input) {
  // Map of attribute value ids of the domain sub-tree to the new value (from the resource)
  const updates = {};

  // Set of action names to be executed to update the resource based on the new domain
  const actions = new Set();

  for (const value of Object.values(arg)) {
    // Updates domain to fit the new resource
    updates[value.domain.id] = value.normalizedResource;

    actions.add("delete");
    actions.add("create");
  }

  return {
    updates,
    actions: Array.from(actions),
  };
}
