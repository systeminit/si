import uuidv4 from "uuid/v4";
import { OperatingSystemComponent } from "@/generated/graphql";

interface OperatingSystemArgs {
  id?: string;
  name?: string;
  description?: string;
  operatingSystemName?: string;
  operatingSystemVersion?: string;
  operatingSystemRelease?: string;
  platform?: string;
  platformVersion?: string;
  platformRelease?: string;
}

interface OperatingSystemResult {
  id: string;
  name: string;
  component: OperatingSystemComponent;
  constraints: OperatingSystemArgs;
}

export async function OperatingSystem(args: OperatingSystemArgs): Promise<OperatingSystemResult> {
  return {
    id: uuidv4(),
    name: "dodo",
    constraints: args, 
  }
}
