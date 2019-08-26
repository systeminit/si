import { Integration } from "@/datalayer/integration";

export interface Component {
  id: string;
  name: string;
  description: string;
  rawDataJson: string;
  integration: Integration;
  nodeType: string;
  __typename: string;
}
