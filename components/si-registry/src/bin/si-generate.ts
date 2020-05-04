import chalk from "chalk";
import figlet from "figlet";
import path from "path";
import program from "commander";
import { registry } from "../registry";
import { ProtobufFormatter } from "../codegen/protobuf";
import { CodegenRust } from "../codegen/rust";
import Listr, { ListrRendererValue } from "listr";
import "../loader";
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
      {
        title: `Generating ${chalk.keyword("yellow")("Javascript Library")}`,
        task: (): Listr => {
          return generateJavascriptLibrary();
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

function generateJavascriptLibrary(): Listr {
  const tasks = [];
  tasks.push({
    title: `Javascript library for si-registry`,
    task: async () => {
      await execCmd("npm run build");
    },
  });
  return new Listr(tasks, { concurrent: true });
}

function generateProtobuf(): Listr {
  const tasks = [];
  for (const serviceName of registry.serviceNames()) {
    tasks.push({
      title: `Protobuf Service ${chalk.keyword("darkseagreen")(serviceName)}`,
      task: async () => {
        const cp = new ProtobufFormatter(
          registry.getObjectsForServiceName(serviceName),
        );
        const protoFile = path.join("./proto", `si.${serviceName}.proto`);
        const writeFileAsync = promisify(fs.writeFile);
        await writeFileAsync(protoFile, cp.generateString());
      },
    });
  }
  return new Listr(tasks, { concurrent: true });
}

function generateRust(): Listr {
  const tasks = [];

  for (const serviceName of registry.serviceNames()) {
    const codegenRust = new CodegenRust(serviceName);
    const systemObjects = registry.getObjectsForServiceName(serviceName);

    tasks.push({
      title: `Rust service ${chalk.keyword("orange")(
        "gen/service.rs",
      )} for ${serviceName}`,
      task: async (): Promise<void> => {
        await codegenRust.generateGenService();
      },
    });

    if (systemObjects.some(o => o.kind() != "baseObject")) {
      tasks.push({
        title: `Rust ${chalk.keyword("orange")(
          "gen/mod.rs",
        )} for ${serviceName}`,
        task: async (): Promise<void> => {
          await codegenRust.generateGenMod();
        },
      });

      tasks.push({
        title: `Rust ${chalk.keyword("orange")(
          "gen/model/mod.rs",
        )} for ${serviceName}`,
        task: async (): Promise<void> => {
          await codegenRust.generateGenModelMod();
        },
      });

      for (const systemObject of registry.getObjectsForServiceName(
        serviceName,
      )) {
        if (systemObject.kind() != "baseObject") {
          tasks.push({
            title: `Rust model ${chalk.keyword("orange")(serviceName)} ${
              systemObject.typeName
            }`,
            task: async (): Promise<void> => {
              await codegenRust.generateGenModel(systemObject);
            },
          });
        }
      }

      tasks.push({
        title: `Rust format ${serviceName}`,
        task: async (): Promise<void> => {
          await codegenRust.formatCode();
        },
      });
    }
  }

  return new Listr(tasks, { concurrent: false });
}
