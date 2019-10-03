import faker from "faker";
import util from "util";

import typeDefs from "./schema.graphql";
import { createComponentModule } from "@/modules/base";

import { GqlRoot, GqlContext, GqlInfo } from "@/app.module";
import { SshKey, SshKeyEntity } from "@/datalayer/component/ssh-key";
import { checkAuthentication } from "@/modules/auth";

import { UserInputError } from "apollo-server";

interface CreateSshKeyPayload {
  sshKey: SshKeyEntity;
}

interface SshKeyGen {
  publicKey: string;
  privateKey: string;
  bubbleBabble: string;
  fingerPrint: string;
  randomArt: string;
}

async function runSshKeyFingerprints(
  privateKey: string,
  publicKey: string,
): Promise<SshKeyGen> {
  const execFile = util.promisify(require("child_process").execFile);
  const writeFile = util.promisify(require("fs").writeFile);

  await execFile("rm", ["-f", "/tmp/bugbear-key"]);
  await execFile("rm", ["-f", "/tmp/bugbear-key.pub"]);

  await writeFile("/tmp/bugbear-key", privateKey);
  await writeFile("/tmp/bugbear-key.pub", publicKey);

  const { stdout: fingerPrint } = await execFile("ssh-keygen", [
    "-l",
    "-f",
    "/tmp/bugbear-key",
  ]);
  const { stdout: bubbleBabble } = await execFile("ssh-keygen", [
    "-B",
    "-f",
    "/tmp/bugbear-key",
  ]);
  const { stdout: randomArt } = await execFile("ssh-keygen", [
    "-l",
    "-v",
    "-f",
    "/tmp/bugbear-key",
  ]);

  return {
    publicKey: publicKey.trim(),
    privateKey: privateKey.trim(),
    fingerPrint: fingerPrint.trim(),
    bubbleBabble: bubbleBabble.trim(),
    randomArt: randomArt.trim(),
  };
}

async function runSshKeyGen(args: string[]): Promise<SshKeyGen> {
  const execFile = util.promisify(require("child_process").execFile);
  const readFile = util.promisify(require("fs").readFile);

  await execFile("rm", ["-f", "/tmp/bugbear-key"]);
  await execFile("rm", ["-f", "/tmp/bugbear-key.pub"]);

  const { stdout } = await execFile("ssh-keygen", args);
  console.log(stdout);

  const privateKey = await readFile("/tmp/bugbear-key", "utf-8");
  const publicKey = await readFile("/tmp/bugbear-key.pub", "utf-8");
  const { stdout: fingerPrint } = await execFile("ssh-keygen", [
    "-l",
    "-f",
    "/tmp/bugbear-key",
  ]);
  const { stdout: bubbleBabble } = await execFile("ssh-keygen", [
    "-B",
    "-f",
    "/tmp/bugbear-key",
  ]);
  const { stdout: randomArt } = await execFile("ssh-keygen", [
    "-l",
    "-v",
    "-f",
    "/tmp/bugbear-key",
  ]);

  return {
    publicKey: publicKey.trim(),
    privateKey: privateKey.trim(),
    fingerPrint: fingerPrint.trim(),
    bubbleBabble: bubbleBabble.trim(),
    randomArt: randomArt.trim(),
  };
}

async function createEntity(
  _obj: GqlRoot,
  { input: { constraints, args, workspace } },
  _context: GqlContext,
  info: GqlInfo,
): Promise<CreateSshKeyPayload> {
  const user = await checkAuthentication(info);

  const searchValue =
    constraints ||
    JSON.stringify({
      keyType: "RSA",
      bits: 2048,
      keyFormat: "RFC4716",
    });
  const componentList = await SshKey.find(
    { where: { workspace: workspace, search: searchValue } },
    user,
  );
  if (componentList.length > 1) {
    throw new UserInputError(
      `Constraints resolve to ${componentList.length} components; must resolve to 1`,
    );
  }
  const component = componentList[0];

  let name: string;
  let description: string;

  let keyData: SshKeyGen;

  if (args) {
    if (args.name) {
      name = args.name;
    }
    if (args.description) {
      description = args.description;
    }
    if (args.privateKey && args.publicKey) {
      keyData = await runSshKeyFingerprints(args.privateKey, args.publicKey);
    }
  }

  if (!name) {
    name = faker.commerce.productName();
  }
  if (!description) {
    description = name;
  }
  if (!keyData) {
    keyData = await runSshKeyGen([
      "-t",
      component.keyType,
      "-m",
      component.keyFormat,
      "-b",
      `${component.bits}`,
      "-C",
      name,
      "-f",
      "/tmp/bugbear-key",
      "-N",
      '""',
    ]);
  }

  const data: SshKeyEntity = {
    name,
    description,
    comment: name,
    keyType: component.keyType,
    keyFormat: component.keyFormat,
    bits: component.bits,
    fingerPrint: keyData.fingerPrint,
    bubbleBabble: keyData.bubbleBabble,
    randomArt: keyData.randomArt,
    publicKey: keyData.publicKey,
    privateKey: keyData.privateKey,
    userId: user.fqId,
    workspaceId: `workspace:${workspace}`,
    componentId: component.fqId(),
  };

  const sshKeyEntity = SshKeyEntity.New(data);

  await sshKeyEntity.save();
  return { sshKey: sshKeyEntity };
}

export const SshKeys = createComponentModule({
  typeDefs,
  createEntity,
  componentName: "SshKey",
  component: SshKey,
  entity: SshKeyEntity,
});
