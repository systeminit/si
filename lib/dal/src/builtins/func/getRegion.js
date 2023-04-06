function getRegion(input) {
  const defaultName = 'region';
  if (!input.region || input.region.length === 0) {
    return defaultName;
  }

  return input.region;
}