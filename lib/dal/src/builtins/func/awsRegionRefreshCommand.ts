async function refresh(component: Input): Promise<Output> {
  return { value: { region: component.properties.domain.region }, status: "ok" };
}
