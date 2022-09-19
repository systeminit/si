/* eslint-disable import/extensions */
import AwsIconRaw from "~icons/cib/amazon-aws?raw";
import CoreOsIconRaw from "~icons/ri/coreos-fill?raw";
import SiLogoWts from "@/assets/images/si-logo-wts.svg?raw";
import DockerIconRaw from "@/assets/images/3p-logos/docker/docker-icon.svg?raw";
import KubernetesIconRaw from "@/assets/images/3p-logos/kubernetes/kubernetes-icon.svg?raw";

export const LogoIcons: Record<string, string> = {
  aws: AwsIconRaw as unknown as string,
  coreos: CoreOsIconRaw as unknown as string,
  docker: DockerIconRaw,
  kubernetes: KubernetesIconRaw,
  si: SiLogoWts,
};
