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

type RenderProvider = "aws";

export async function renderAsset(props: Array<Prop>, provider: RenderProvider): Promise<string> {
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
  const assetDefinition = eta.render("@assetMain", { props, provider });

  const command = new Deno.Command("deno", {
    args: ["fmt", "-"],
    stdin: "piped",
    stdout: "piped",
    stderr: "piped",
  });
  const running = command.spawn();
  const writer = running.stdin.getWriter();
  await writer.write(new TextEncoder().encode(assetDefinition));
  writer.releaseLock();
  await running.stdin.close();

  const n = await running.stdout.getReader().read();
  const stdout = new TextDecoder().decode(n.value);
  const result = await running.status;
  if (result.success) {
    return stdout;
  } else {
    return assetDefinition;
  }
}
