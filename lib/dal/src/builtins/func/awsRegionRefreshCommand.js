async function refresh(component) {
  return { value: { region: component.properties.domain.region }, status: "ok" };
}
