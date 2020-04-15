#!/usr/bin/env node

import chalk from "chalk";
import figlet from "figlet";
import path from "path";
import program from "commander";
import { registry } from "src/componentRegistry";
import { CodegenProtobuf } from "src/codegen/protobuf";
import { CodegenRust, generateGenMod } from "src/codegen/rust";
import Listr, { ListrRendererValue } from "listr";
import "src/loader";
import fs from "fs";
import { promisify } from "util";
import childProcess from "child_process";
import util from "util";
const execCmd = util.promisify(childProcess.exec);

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
        title: `Generating ${chalk.keyword("darkseagreen")("Protobuf")}`,
        task: (): Listr => {
          return generateProtobuf();
        },
      },
      {
        title: `Generating ${chalk.keyword("orange")("Rust")}`,
        task: (): Listr => {
          return generateRust();
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
      title: `Protobuf ${chalk.keyword("darkseagreen")(
        component.protobufPackageName(),
      )}`,
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

function generateRust(): Listr {
  const tasks = [];
  const writtenComponents: {
    [key: string]: string[];
  } = {};

  for (const component of registry.components) {
    if (component.noStd) {
      continue;
    }
    if (!writtenComponents[component.siPathName]) {
      writtenComponents[component.siPathName] = [];
    }
    writtenComponents[component.siPathName].push(component.typeName);
    tasks.push({
      title: `Rust for ${chalk.keyword("orange")(component.typeName)}`,
      task: (): Listr => {
        return new Listr(
          [
            {
              title: `Rust Component ${chalk.keyword("orange")(
                component.typeName,
              )}`,
              task: async (): Promise<void> => {
                const cp = new CodegenRust(component);
                await cp.generateComponentImpls();
              },
            },
            {
              title: `Rust Component ${chalk.keyword("orange")(
                component.typeName,
              )} mod.rs`,
              task: async (): Promise<void> => {
                const cp = new CodegenRust(component);
                await cp.generateComponentMod();
              },
            },
          ],
          { concurrent: true },
        );
      },
    });
  }
  tasks.push({
    title: `Rust $chalk.keyword("orange")("gen/mod.rs")`,
    task: async (): Promise<void> => {
      await generateGenMod(writtenComponents);
    },
  });

  return new Listr(tasks, { concurrent: true });
}

