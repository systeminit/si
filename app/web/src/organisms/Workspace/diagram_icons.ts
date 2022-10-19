/* eslint-disable import/extensions,import/order */
import AwsIconRaw from "~icons/cib/amazon-aws?raw";
import CoreOsIconRaw from "~icons/ri/coreos-fill?raw";
import SiLogoWts from "@/assets/images/si-logo-wts.svg?raw";
import DockerIconRaw from "@/assets/images/3p-logos/docker/docker-icon.svg?raw";
import KubernetesIconRaw from "@/assets/images/3p-logos/kubernetes/kubernetes-icon.svg?raw";

import MinusCircleIconRaw from "~icons/heroicons-solid/minus-circle?raw";
import PlusCircleIconRaw from "~icons/heroicons-solid/plus-circle?raw";
import TildeCircleIconRaw from "@/assets/images/custom-icons/tilde-circle.svg?raw";

export const DiagramIcons: Record<string, string> = {
  // provider logos
  AWS: AwsIconRaw as unknown as string,
  CoreOS: CoreOsIconRaw as unknown as string,
  Docker: DockerIconRaw,
  Kubernetes: KubernetesIconRaw,
  si: SiLogoWts,

  // change status
  // NOTE - keep these up to date with icons loaded in Icon.vue and selected in StatusIndicatorIcon.vue
  "change-modified": TildeCircleIconRaw,
  "change-deleted": MinusCircleIconRaw as unknown as string,
  "change-added": PlusCircleIconRaw as unknown as string,
};
