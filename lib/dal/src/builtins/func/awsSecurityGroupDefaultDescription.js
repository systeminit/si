function setDefaultDescription(input) {
  const defaultDescription = 'Managed by System Initiative';
  if (!input.name || input.name.length === 0) {
    return defaultDescription;
  }

  return `${input.name} - ${defaultDescription}`;
}