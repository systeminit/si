import { Eta } from "https://deno.land/x/eta@v3.2.0/src/index.ts";
import { Prop } from "./props.ts";
import { partial as assetMainPartial } from "./templates/assetMain.ts";
import { partial as arrayPartial } from "./templates/array.ts";
import { partial as booleanPartial } from "./templates/boolean.ts";
import { partial as mapPartial } from "./templates/map.ts";
import { partial as numberPartial } from "./templates/number.ts";
import { partial as objectPartial } from "./templates/object.ts";
import { partial as stringPartial } from "./templates/string.ts";
import { partial as renderPropPartial } from "./templates/renderProp.ts";
import { partial as codeGenMainPartial } from "./templates/codeGenMain.ts";
import { partial as createMainPartial } from "./templates/createMain.ts";
import { partial as refreshMainPartial } from "./templates/refreshMain.ts";
import { partial as deleteMainPartial } from "./templates/deleteMain.ts";
import { ArgInput, ArgOutput } from "./resource_generator.ts";

type RenderProvider = "aws";

export function useEta(): Eta {
  const eta = new Eta({
    debug: true,
    autoEscape: false,
  });
  eta.loadTemplate("@assetMain", assetMainPartial);
  eta.loadTemplate("@arrayPartial", arrayPartial);
  eta.loadTemplate("@booleanPartial", booleanPartial);
  eta.loadTemplate("@mapPartial", mapPartial);
  eta.loadTemplate("@numberPartial", numberPartial);
  eta.loadTemplate("@objectPartial", objectPartial);
  eta.loadTemplate("@stringPartial", stringPartial);
  eta.loadTemplate("@renderPropPartial", renderPropPartial);
  eta.loadTemplate("@codeGenMain", codeGenMainPartial);
  eta.loadTemplate("@createMain", createMainPartial);
  eta.loadTemplate("@refreshMain", refreshMainPartial);
  eta.loadTemplate("@deleteMain", deleteMainPartial);
  return eta;
}

export async function fmt(ts: string): Promise<string> {
  const command = new Deno.Command("deno", {
    args: ["fmt", "-"],
    stdin: "piped",
    stdout: "piped",
    stderr: "piped",
  });
  const running = command.spawn();
  const writer = running.stdin.getWriter();
  await writer.write(new TextEncoder().encode(ts));
  writer.releaseLock();
  await running.stdin.close();

  const n = await running.stdout.getReader().read();
  const stdout = new TextDecoder().decode(n.value);
  const result = await running.status;
  if (result.success) {
    return stdout;
  } else {
    return ts;
  }
}

export async function renderAsset(props: Array<Prop>, provider: RenderProvider): Promise<string> {
  const eta = useEta();
  const assetDefinition = eta.render("@assetMain", { props, provider });
  return await fmt(assetDefinition);
}

export async function renderCodeGen(provider: RenderProvider): Promise<string> {
  const eta = useEta();
  const codeGen = eta.render("@codeGenMain", { provider });
  return await fmt(codeGen);
}

export interface RenderCreateBase {
  provider: RenderProvider
}

export interface RenderCreateAws extends RenderCreateBase {
  provider: "aws";
  awsService: string;
  awsCommand: string;
}

export type RenderCreateOptions = RenderCreateAws;

export async function renderCreate(options: RenderCreateOptions): Promise<string> {
  const eta = useEta();
  const create = eta.render("@createMain", { options });
  return await fmt(create);
}

export interface RenderRefreshOptions {
  provider: "aws";
  awsService: string;
  awsCommand: string;
  inputs: Array<ArgInput>;
  outputs: Array<ArgOutput>;
};

export async function renderRefresh(options: RenderRefreshOptions): Promise<string> {
  const eta = useEta();
  const refresh = eta.render("@refreshMain", options);
  return await fmt(refresh);
}

export interface RenderDeleteOptions {
  provider: "aws";
  awsService: string;
  awsCommand: string;
  inputs: Array<ArgInput>;
}

export async function renderDelete(options: RenderDeleteOptions): Promise<string> {
  const eta = useEta();
  const deletetemp = eta.render("@deleteMain", options);
  return await fmt(deletetemp);
}

