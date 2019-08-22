import { Integration } from "@/datalayer/integration";

export interface ServerCpu {
  id: string;
  name: string;
  cores: number;
  baseFreqMHz: number;
  allCoreTurboFreqMHz: number;
  singleCoreTurboFreqMHz: number;
}

export interface ServerComponent {
  id: string;
  name: string;
  description: string;
  rawDataJson: string;
  integration: Integration;
  cpu: ServerCpu;
  memoryGIB: number;
  nodeType: string;
  __typename: string;
}

export async function serverComponentData(): Promise<ServerComponent[]> {
  let aws = await Integration.query().findById(1);
  let gcp = await Integration.query().findById(3);
  let serverComponentData: ServerComponent[] = [
    {
      id: "1",
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
      },
      memoryGIB: 8,
      __typename: "ServerComponent",
    },
    {
      id: "2",
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
      },
      __typename: "ServerComponent",
    },
    {
      id: "3",
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
      },
      __typename: "ServerComponent",
    },
    {
      id: "4",
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
      },
      __typename: "ServerComponent",
    },
  ];
  return serverComponentData;
}
