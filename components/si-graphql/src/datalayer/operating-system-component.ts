import { Component } from "@/datalayer/component";
import { Integration } from "@/datalayer/integration";

enum OperatingSystemAction {
  START = "START",
  STOP = "STOP",
  RESTART = "RESTART",
  UPDATE = "UPDATE",
  CONFIGURE = "CONFIGURE",
}

export interface OperatingSystemComponent extends Component {
  operatingSystemName: string;
  operatingSystemVersion: string;
  operatingSystemRelease: string;
  platform: string;
  platformVersion: string;
  platformRelease: string;
  architecture: string[];
  operatingSystemSupportedActions: OperatingSystemAction[];
  __typename: string;
}

// us-west-1	bionic	18.04 LTS	amd64	hvm:ebs-ssd	20190814	ami-056d04da775d124d7	hvm
//
//rawDataJson: '{"ami-id": "ami-056d04da775d124d7", "aki-id": "hvm", "release": "20190814" }',
//
export async function operatingSystemComponentData(): Promise<
  OperatingSystemComponent[]
> {
  const globalIntegration = await Integration.query()
    .where("name", "Global")
    .first();
  const operatingSystemComponentData: OperatingSystemComponent[] = [
    {
      id: "a1c8618e-ffab-4d7e-be0c-ba50cf88a63f",
      name: "Ubuntu 18.04.3 LTS",
      description: "Ubuntu 18.04 LTS (Bionic Beaver)",
      rawDataJson: '{ "lts": true }',
      integration: globalIntegration,
      nodeType: "Operating System",
      operatingSystemName: "Linux",
      operatingSystemVersion: "4.15.0",
      operatingSystemRelease: "1040",
      platform: "ubuntu",
      platformVersion: "18.04",
      platformRelease: "1",
      architecture: ["x86-64", "i386", "armhf", "arm64", "ppc64"],
      operatingSystemSupportedActions: [
        OperatingSystemAction.START,
        OperatingSystemAction.STOP,
        OperatingSystemAction.RESTART,
        OperatingSystemAction.UPDATE,
        OperatingSystemAction.CONFIGURE,
      ],
      __typename: "OperatingSystemComponent",
    },
    {
      id: "deb233a1-ba65-437b-9506-27918eae363c",
      name: "Arch Linux 2019.08.01",
      description: "Arch Linux 2019.08.01",
      rawDataJson: "{}",
      integration: globalIntegration,
      nodeType: "Operating System",
      operatingSystemName: "Linux",
      operatingSystemVersion: "5.2.5",
      operatingSystemRelease: "arch1-1",
      platform: "arch",
      platformVersion: "2019.08.01",
      platformRelease: "1",
      architecture: ["x86-64"],
      operatingSystemSupportedActions: [
        OperatingSystemAction.START,
        OperatingSystemAction.STOP,
        OperatingSystemAction.RESTART,
        OperatingSystemAction.UPDATE,
      ],
      __typename: "OperatingSystemComponent",
    },
    {
      id: "80c90b9c-6aa3-422e-8ed9-6347168d289a",
      name: "Microsoft Windows Server 2019",
      description: "Microsoft Windows Server 2019",
      rawDataJson: "{}",
      integration: globalIntegration,
      nodeType: "Operating System",
      operatingSystemName: "Windows",
      operatingSystemVersion: "10.0.17763",
      operatingSystemRelease: "1",
      platform: "Windows",
      platformVersion: "10.0.17763",
      platformRelease: "1",
      architecture: ["x86-64"],
      operatingSystemSupportedActions: [
        OperatingSystemAction.START,
        OperatingSystemAction.STOP,
        OperatingSystemAction.RESTART,
        OperatingSystemAction.UPDATE,
      ],
      __typename: "OperatingSystemComponent",
    },
  ];
  return operatingSystemComponentData;
}
