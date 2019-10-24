export interface IntegrationInstance {
  id: String;
  workspaces: Workspace[];
}

export interface Workspace {
  id: String;
  integrationInstances: IntegrationInstance[];
}
