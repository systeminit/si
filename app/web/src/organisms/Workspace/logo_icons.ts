/* eslint-disable import/extensions */
import CoreOsIconRaw from "~icons/ri/coreos-fill?raw";
import AwsAmiIconRaw from "@/assets/images/external-icons/aws-ami-light.svg?raw";
import AwsEc2IconRaw from "@/assets/images/external-icons/aws-ec2-light.svg?raw";
import DockerIconRaw from "@/assets/images/3p-logos/docker/docker-icon.svg?raw";
import KubernetesIconRaw from "@/assets/images/3p-logos/kubernetes/kubernetes-icon.svg?raw";

export const LogoIcons: Record<string, string> = {
  awsAmi: AwsAmiIconRaw,
  awsEc2: AwsEc2IconRaw,
  coreos: CoreOsIconRaw as unknown as string,
  docker: DockerIconRaw,
  kubernetes: KubernetesIconRaw,
};
