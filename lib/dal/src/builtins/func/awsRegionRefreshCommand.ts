async function refresh(component: Input): Promise<Output> {
  return { payload: { region: component.properties.domain.region }, status: "ok" };
}
