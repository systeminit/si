#!/usr/bin/env node

import chalk from "chalk";
import figlet from "figlet";
import path from "path";
import program from "commander";
import { registry } from "@/componentRegistry";
import { CodegenProtobuf } from "@/codegen/protobuf";
import Listr, { ListrRendererValue } from "listr";
import "@/loader";
import fs from "fs";
import { promisify } from "util";

console.log(
  chalk.greenBright(figlet.textSync("Lets go!", { horizontalLayout: "full" })),
);

program
  .version("0.0.1")
  .description("Code Generation to rule them all")
  .option("-v, --verbose", "show verbose output")
  .parse(process.argv);

main(program);

function main(program: program.Command): void {
  // @ts-ignore
  let renderer: ListrRendererValue<any>;
  if (program.verbose) {
    renderer = "verbose";
  } else {
    renderer = "default";
  }
  const tasks = new Listr(
    [
      {
        title: `Generating ${chalk.yellowBright("Protobuf")}`,
        task: (): Listr => {
          return generateProtobuf();
        },
      },
    ],
    {
      renderer,
      concurrent: true,
    },
  );
  tasks.run().catch((err: Error): void => {
    console.log(err);
  });
}

function generateProtobuf(): Listr {
  const tasks = [];
  for (const component of registry.components) {
    tasks.push({
      title: `Protobuf for ${chalk.blue(component.typeName)}`,
      task: async () => {
        const cp = new CodegenProtobuf(component);
        const protoFile = path.join(
          __dirname,
          "..",
          "..",
          "proto",
          `si.${component.protobufPackageName()}.proto`,
        );
        const writeFileAsync = promisify(fs.writeFile);
        await writeFileAsync(protoFile, cp.generateString());
      },
    });
  }
  return new Listr(tasks, { concurrent: true });
}
