async function qualificationAwsRegionHasRegionSet(
  input: Input,
): Promise<Output> {
  const isRegionSet = (input.domain.region?.trim().length ?? 0) > 0;

  return {
    result: isRegionSet ? "success" : "failure",
    message: isRegionSet
      ? undefined
      : "A valid AWS region must be set on this region frame",
  };
}
