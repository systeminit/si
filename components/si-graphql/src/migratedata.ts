import { Integration } from "@/datalayer/integration";
import { Cpu } from "@/datalayer/component/cpu";
import { OperatingSystem } from "@/datalayer/component/operating-system";
import { Server } from "@/datalayer/component/server";
import { DiskImage } from "@/datalayer/component/disk-image";
import "@/datalayer/component/relationships";
import { getPortData } from "@/migrate/ports";
import { getSshKeyData } from "@/migrate/ssh-key";

async function importPortData(): Promise<void> {
  const data = await getPortData();
  for (const d of data) {
    console.log(`Migrating ${d.fqId()} ${d.name}`);
    await d.save();
  }
}

async function importSshKeyData(): Promise<void> {
  const data = await getSshKeyData();
  for (const d of data) {
    console.log(`Migrating ${d.fqId()} ${d.name}`);
    await d.save();
  }
}

async function importDiskImageData(): Promise<void> {
  const awsIntegration = await Integration.getByName("AWS");
  const ubuntu1804 = await OperatingSystem.getByName("Ubuntu 18.04.3");
  const data = [
    DiskImage.New({
      id: "465fd3e5-fc68-423d-a1c6-ea7a4662d5f1",
      name: "AWS us-west-1 Ubuntu 18.04.03 hvm-instance",
      description: "AWS us-west-1 Ubuntu 18.04.03 hvm-instance 20190814",
      rawDataJson: "{}",
      supportedActions: ["Bundle"],
      integrationId: awsIntegration.fqId,
      format: "ami",
      operatingSystemId: ubuntu1804.fqId(),
    }),
  ];
  for (const d of data) {
    console.log(`Migrating ${d.fqId()} ${d.name}`);
    await d.save();
  }
}

async function importServerData(): Promise<void> {
  const awsIntegration = await Integration.getByName("AWS");
  const cpuIntel8175 = await Cpu.getByName("Intel Xeon 8175");
  const data = [
    Server.New({
      id: "92cd05e5-ca2f-4618-bd9f-fc1fb7a7cb22",
      name: "AWS EC2 m5.large",
      description: "AWS EC2 m5.large",
      rawDataJson: "{}",
      integrationId: awsIntegration.fqId,
      cpuId: cpuIntel8175.fqId(),
      memoryGIB: 8,
      cpuCores: 2,
      supportedActions: ["Start", "Stop", "Restart", "Create", "Destroy"],
    }),
  ];
  for (const d of data) {
    console.log(`Migrating ${d.fqId()} ${d.name}`);
    await d.save();
  }
}

async function importOperatingSystemData(): Promise<void> {
  const globalIntegration = await Integration.getByName("Global");
  const data = [
    OperatingSystem.New({
      id: "a1c8618e-ffab-4d7e-be0c-ba50cf88a63f",
      name: "Ubuntu 18.04.3",
      description: "Ubuntu 18.04 LTS (Bionic Beaver)",
      rawDataJson: '{ "lts": true }',
      integrationId: globalIntegration.fqId,
      operatingSystemName: "Linux",
      operatingSystemVersion: "4.15.0",
      operatingSystemRelease: "1040",
      platform: "ubuntu",
      platformVersion: "18.04",
      platformRelease: "1",
      architecture: ["x86-64", "i386", "armhf", "arm64", "ppc64"],
      supportedActions: ["Start", "Stop", "Restart", "Update", "Configure"],
    }),
  ];
  for (const d of data) {
    console.log(`Migrating ${d.fqId()} ${d.name}`);
    await d.save();
  }
}

async function importCpuData(): Promise<void> {
  const globalIntegration = await Integration.getByName("Global");
  const data = [
    Cpu.New({
      id: "c391458b-9394-48bd-8fbb-1b807ece5e56",
      description: "Intel Xeon 8175 CPU",
      rawDataJson: "{}",
      integrationId: globalIntegration.fqId,
      name: "Intel Xeon 8175",
      cores: 2,
      baseFreqMHz: 3100,
      allCoreTurboFreqMHz: 3100,
      singleCoreTurboFreqMHz: 3100,
      architecture: "x86-64",
      manufacturer: "Intel",
      supportedActions: [],
    }),
  ];
  for (const d of data) {
    console.log(`Migrating ${d.fqId()} ${d.name}`);
    await d.save();
  }
}

async function importIntegrationData(): Promise<void> {
  const data = [
    new Integration({
      id: "0d196d4c-c441-4a28-ad74-550593615c9f",
      name: "AWS",
      description: "Amazon Web Services",
      options: {
        fields: [
          {
            id: "access_key",
            name: "Access Key",
            type: "input",
          },
          {
            id: "secret_key",
            name: "Secret Key",
            type: "secret",
          },
        ],
      },
      image: "aws.png",
    }),
    new Integration({
      id: "bc11cdf9-8ce3-4af3-9df7-a17289026dc6",
      name: "Azure",
      description: "Microsoft Azure",
      options: {
        fields: [
          {
            id: "id_token",
            name: "ID Token",
            type: "input",
          },
        ],
      },
      image: "azure.png",
    }),
    new Integration({
      id: "d085f0d8-5d21-42b5-a745-037d4e1c9523",
      name: "Google",
      description: "Google Cloud",
      options: {
        fields: [
          {
            id: "google_json",
            name: "Google Token",
            type: "input",
          },
        ],
      },
      image: "google.png",
    }),
    new Integration({
      id: "4982d1ee-1391-4ae0-864a-4d55d1f8b0b7",
      name: "GitHub",
      description: "GitHub",
      options: {
        fields: [
          {
            id: "github_token",
            name: "GitHub Token",
            type: "input",
          },
        ],
      },
      image: "github.png",
    }),
    new Integration({
      id: "9bfc0c3e-6273-4196-8e74-364761cb1b04",
      name: "Global",
      description: "Global",
      options: {},
      image: "global.svg",
    }),
  ];
  for (const d of data) {
    console.log(`Migrating ${d.fqId} ${d.name}`);
    await d.upsert();
  }
}

async function main(): Promise<void> {
  console.log("---- Migrating Data ----");
  try {
    console.log("** Integrations");
    await importIntegrationData();
    console.log("** CPU");
    await importCpuData();
    console.log("** Operating System");
    await importOperatingSystemData();
    console.log("** Server");
    await importServerData();
    console.log("** Disk Image");
    await importDiskImageData();
    console.log("** Ports");
    await importPortData();
    console.log("** SSH Keys");
    await importSshKeyData();
  } catch (e) {
    console.log("Failed: ", e);
    process.exit();
  }
  console.log("----Finished----");
  process.exit();
}

main();
