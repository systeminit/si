/* eslint-disable import/extensions,import/order */
import AwsIconRaw from "~icons/cib/amazon-aws?raw";
import CoreOsIconRaw from "~icons/ri/coreos-fill?raw";
import SiLogoWts from "@/assets/images/si-logo-wts.svg?raw";
import DockerIconRaw from "@/assets/images/3p-logos/docker/docker-icon.svg?raw";
import KubernetesIconRaw from "@/assets/images/3p-logos/kubernetes/kubernetes-icon.svg?raw";

export const DiagramIcons: Record<string, string> = {
  // provider logos
  AWS: AwsIconRaw as unknown as string,
  CoreOS: CoreOsIconRaw as unknown as string,
  Docker: DockerIconRaw,
  Kubernetes: KubernetesIconRaw,
  si: SiLogoWts,
};
