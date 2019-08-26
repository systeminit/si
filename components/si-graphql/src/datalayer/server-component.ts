import { Component } from "@/datalayer/component";
import { Integration } from "@/datalayer/integration";

export interface ServerCpu {
  id: string;
  name: string;
  cores: number;
  baseFreqMHz: number;
  allCoreTurboFreqMHz: number;
  singleCoreTurboFreqMHz: number;
  architecture: string;
}

enum ServerAction {
  START = "START",
  STOP = "STOP",
  RESTART = "RESTART",
  DESTROY = "DESTROY",
  CREATE = "CREATE",
  CONSOLE = "CONSOLE",
}

export interface ServerComponent extends Component {
  cpu: ServerCpu;
  memoryGIB: number;
  serverSupportedActions: ServerAction[];
}

export async function serverComponentData(): Promise<ServerComponent[]> {
  let aws = await Integration.query()
    .where("name", "AWS")
    .first();
  let gcp = await Integration.query()
    .where("name", "Google")
    .first();
  let serverComponentData: ServerComponent[] = [
    {
      id: "92cd05e5-ca2f-4618-bd9f-fc1fb7a7cb22",
      name: "m5.large",
      description: "AWS EC2 m5.large",
      rawDataJson: "{}",
      integration: aws,
      nodeType: "Server",
      cpu: {
        id: "1",
        name: "Intel Xeon 8175",
        cores: 2,
        baseFreqMHz: 3100,
        allCoreTurboFreqMHz: 3100,
        singleCoreTurboFreqMHz: 3100,
        architecture: "x86-64",
      },
      memoryGIB: 8,
      serverSupportedActions: [
        ServerAction.START,
        ServerAction.STOP,
        ServerAction.RESTART,
        ServerAction.CREATE,
        ServerAction.DESTROY,
      ],
      __typename: "ServerComponent",
    },
    {
      id: "8e875463-0046-49df-b75b-6086660827ca",
      name: "m5.xlarge",
      description: "AWS EC2 m5.xlarge",
      rawDataJson: "{}",
      integration: aws,
      nodeType: "Server",
      memoryGIB: 16,
      cpu: {
        id: "1",
        name: "Intel Xeon 8175",
        cores: 4,
        baseFreqMHz: 3100,
        allCoreTurboFreqMHz: 3100,
        singleCoreTurboFreqMHz: 3100,
        architecture: "x86-64",
      },
      serverSupportedActions: [
        ServerAction.START,
        ServerAction.STOP,
        ServerAction.RESTART,
        ServerAction.CREATE,
        ServerAction.DESTROY,
      ],
      __typename: "ServerComponent",
    },
    {
      id: "3f42eb0b-8500-4ef3-a8ce-6e2b114f6c07",
      name: "n1-standard-1",
      description: "GCP n1-standard-1",
      rawDataJson: "{}",
      integration: gcp,
      nodeType: "Server",
      memoryGIB: 16,
      cpu: {
        id: "2",
        name: "Intel Xeon Scalable Processor (Skylake)",
        cores: 4,
        baseFreqMHz: 2000,
        allCoreTurboFreqMHz: 2700,
        singleCoreTurboFreqMHz: 3500,
        architecture: "x86-64",
      },
      serverSupportedActions: [
        ServerAction.START,
        ServerAction.STOP,
        ServerAction.RESTART,
        ServerAction.CREATE,
        ServerAction.DESTROY,
      ],
      __typename: "ServerComponent",
    },
    {
      id: "08430184-fa01-4a87-a439-9341c1d184ec",
      name: "n1-standard-1",
      description: "GCP n1-standard-1",
      rawDataJson: "{}",
      integration: gcp,
      nodeType: "Server",
      memoryGIB: 16,
      cpu: {
        id: "3",
        name: "Intel Xeon E7 (Broadwell E7)",
        cores: 4,
        baseFreqMHz: 2200,
        allCoreTurboFreqMHz: 2800,
        singleCoreTurboFreqMHz: 3800,
        architecture: "x86-64",
      },
      serverSupportedActions: [
        ServerAction.START,
        ServerAction.STOP,
        ServerAction.RESTART,
        ServerAction.CREATE,
        ServerAction.DESTROY,
      ],
      __typename: "ServerComponent",
    },
  ];
  return serverComponentData;
}
