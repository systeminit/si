import { graphqlMutation } from "./apollo";
import { KubernetesDeploymentEntity } from "./graphql-types";

interface KubernetesDeploymentEntityConstructor {
  name: string;
  workspaceId: string;
  changeSetId: string;
}

export class KubernetesDeploymentEntityFactory {
  name: string;
  workspaceId: string;
  changeSetId: string;

  constructor(args: KubernetesDeploymentEntityConstructor) {
    this.name = args.name;
    this.workspaceId = args.workspaceId;
    this.changeSetId = args.changeSetId;
  }

  async create(count?: number): Promise<KubernetesDeploymentEntity> {
    const variables = {
      name: `${this.name}${count ? count : ""}`,
      displayName: `${this.name}${count ? count : ""}`,
      description: `really ${this.name}${count ? count : ""}`,
      workspaceId: this.workspaceId,
      changeSetId: this.changeSetId,
      properties: {
        kubernetesObject: {
          kind: "your butt",
          apiVersion: "1.0",
        },
      },
      constraints: { kubernetesVersion: "V1_15" },
    };
    console.log("creating kde", { variables });
    const result = await graphqlMutation({
      typeName: "kubernetesDeploymentEntity",
      methodName: "create",
      variables,
    });
    return result["item"];
  }
}

