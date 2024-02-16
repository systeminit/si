import { Prop, PropParent } from "./props.ts";
import { camelCase } from "https://deno.land/x/case/mod.ts";
import { singular } from "https://deno.land/x/deno_plural/mod.ts";

type AwsScaffold = Record<string, unknown>;

export function awsGenerate(
  awsService: string,
  awsCommand: string,
): Array<Prop> {
  const scaffold = getAwsCliScaffold(awsService, awsCommand);
  const props = propsFromScaffold(scaffold, []);
  return props;
}

function getAwsCliScaffold(
  awsService: string,
  awsCommand: string,
): AwsScaffold {
  const command = new Deno.Command("aws", {
    args: [awsService, awsCommand, "--generate-cli-skeleton"],
    stdin: "null",
    stdout: "piped",
    stderr: "piped",
  });
  const { code, stdout: rawStdout, stderr: rawStderr } = command.outputSync();
  const stdout = new TextDecoder().decode(rawStdout);
  const stderr = new TextDecoder().decode(rawStderr);

  if (code !== 0) {
    console.error(`AWS cli failed with exit code: ${code}`);
    console.error(`STDOUT:\n\n${stdout.toLocaleString()}`);
    console.error(`STDERR:\n\n${stderr.toLocaleString()}`);
    throw new Error("aws cli command failed");
  }
  const result = JSON.parse(stdout);
  return result;
}

function propsFromScaffold(
  scaffold: AwsScaffold,
  props: Array<Prop>,
  parent?: PropParent,
): Array<Prop> {
  for (let [key, value] of Object.entries(scaffold)) {
    if (
      key == "KeyName" && parent?.kind == "object" &&
      parent?.children.length == 0
    ) {
      // @ts-ignore we know you can't do this officialy, but unofficialy, suck
      // it.
      parent.kind = "map";
      key = singular(parent.name);
    }
    let prop: Prop | undefined;
    if (typeof value === "string") {
      prop = {
        kind: "string",
        name: key,
        variableName: camelCase(`${key}Prop`),
      };
    } else if (typeof value === "number") {
      prop = {
        kind: "number",
        name: key,
        variableName: camelCase(`${key}Prop`),
      };
    } else if (typeof value === "boolean") {
      prop = {
        kind: "boolean",
        name: key,
        variableName: camelCase(`${key}Prop`),
      };
    } else if (Array.isArray(value)) {
      prop = {
        kind: "array",
        name: key,
        variableName: camelCase(`${key}Prop`),
      };
      const childObject: AwsScaffold = {};
      childObject[`${key}Child`] = value[0];
      propsFromScaffold(childObject, props, prop);
    } else if (value == null) {
      // Sometimes the default value is null, and not the empty string. This
      // seems like a reasonable default, even if it is going to be weird.
      prop = {
        kind: "string",
        name: key,
        variableName: camelCase(`${key}Prop`),
      };
    } else if (typeof value === "object") {
      prop = {
        kind: "object",
        name: key,
        variableName: camelCase(`${key}Prop`),
        children: [],
      };
      propsFromScaffold(value as AwsScaffold, props, prop);
    }
    if (prop && parent?.kind == "object") {
      parent.children.push(prop);
    } else if (prop && parent?.kind == "array" || parent?.kind == "map") {
      parent.entry = prop;
    } else if (prop && !parent) {
      props.push(prop);
    }
  }
  return props;
}
