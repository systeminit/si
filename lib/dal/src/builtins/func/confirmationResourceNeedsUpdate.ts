async function needsUpdate(input: Input): Promise<Output> {
  if (input.resource?.value && !_.isEqual(input.domain, input.applied_model?.domain) && !input.deleted_at) {
    return {
      success: false,
      recommendedActions: ["delete", "create"]
    };
  }
  return {
    success: true,
    recommendedActions: [],
  };
}
