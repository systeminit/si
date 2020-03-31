import { SiComponent } from "@/registry/siComponent";
import { sshKey } from "@/registry/sshKey";
import { awsEksClusterRuntime } from "@/registry/awsEksClusterRuntime";
import { kubernetesDeployment } from "@/registry/kubernetes/deployment";

export class SiComponentRegistry {
  siComponents: { [key: string]: SiComponent };

  constructor(siComponents: SiComponent[]) {
    this.siComponents = {};
    for (let siComponent of siComponents) {
      this.siComponents[siComponent.typeName] = siComponent;
    }
  }

  lookup(siComponent: string): SiComponent {
    if (!this.siComponents[siComponent]) {
      throw `Unknown component ${siComponent} in the SiRegsitry`;
    }
    return this.siComponents[siComponent];
  }

  list(): SiComponent[] {
    let result = [];
    for (let siKey of Object.keys(this.siComponents).sort()) {
      result.push(this.siComponents[siKey]);
    }
    return result;
  }
}

export const siComponentRegistry = new SiComponentRegistry([
  sshKey,
  awsEksClusterRuntime,
  kubernetesDeployment,
]);
