import { Component } from "src/component";
import { PropObject } from "src/components/prelude";

import { snakeCase } from "change-case";
import ejs from "ejs";
import fs from "fs";
import path from "path";
import childProcess from "child_process";
import util from "util";

const execCmd = util.promisify(childProcess.exec);

export class CodegenRust {
  component: Component;
  formatter: RustFormatter;

  constructor(component: Component) {
    this.component = component;
    this.formatter = new RustFormatter(component);
  }

  async writeCode(part: string, code: string): Promise<void> {
    const createdPath = await this.makePath();
    const codeFilename = path.join(createdPath, `${snakeCase(part)}.rs`);
    await fs.promises.writeFile(codeFilename, code);
    await execCmd(`rustfmt ${codeFilename}`);
  }

  async makePath(): Promise<string> {
    const pathName = path.join(
      __dirname,
      "..",
      "..",
      "..",
      this.component.siPathName,
      "src",
      "gen",
      snakeCase(this.component.typeName),
    );
    const absolutePathName = path.resolve(pathName);
    await fs.promises.mkdir(path.resolve(pathName), { recursive: true });
    return absolutePathName;
  }

  async generateComponentImpls(): Promise<void> {
    const output = ejs.render(
      "<%- include('rust/component.rs.ejs', { component: component }) %>",
      {
        component: this.component,
        fmt: this.formatter,
      },
      {
        filename: __filename,
      },
    );
    await this.writeCode("component", output);
  }

  async generateComponentMod(): Promise<void> {
    const mods = ["component"];
    const lines = ["// Auto-generated code!", "// No Touchy!\n"];
    for (const mod of mods) {
      lines.push(`pub mod ${mod};`);
    }
    await this.writeCode("mod", lines.join("\n"));
  }
}

export class RustFormatter {
  component: Component;

  constructor(component: Component) {
    this.component = component;
  }

  componentTypeName(): string {
    return snakeCase(this.component.typeName);
  }

  componentOrderByFields(): string {
    const orderByFields = [];
    const componentObject = this.component.asComponent();
    for (const p of componentObject.properties.attrs) {
      if (p.hidden) {
        continue;
      }
      if (p.name == "storable") {
        orderByFields.push('"storable.naturalKey"');
        orderByFields.push('"storable.typeName"');
      } else if (p.name == "siProperties") {
        continue;
      } else if (p.name == "constraints" && p.kind() == "object") {
        // @ts-ignore trust us - we checked
        for (const pc of p.properties.attrs) {
          if (pc.kind() != "object") {
            orderByFields.push(`"constraints.${pc.name}"`);
          }
        }
      } else {
        orderByFields.push(`"${p.name}"`);
      }
    }
    return `vec![${orderByFields.join(",")}]\n`;
  }

  componentImports(): string {
    const result = [];
    result.push(
      `pub use crate::protobuf::${snakeCase(this.component.typeName)}::{`,
      `  Constraints,`,
      `  ListComponentsReply,`,
      `  ListComponentsRequest,`,
      `  PickComponentRequest,`,
      `  Component,`,
      `};`,
    );
    return result.join("\n");
  }

  componentValidation(): string {
    return this.genValidation(this.component.asComponent());
  }

  genValidation(propObject: PropObject): string {
    const result = [];
    for (const prop of propObject.properties.attrs) {
      if (prop.required) {
        const propName = snakeCase(prop.name);
        result.push(`if self.${propName}.is_none() {
          return Err(DataError::ValidationError("missing required ${propName} value".into()));
        }`);
      }
    }
    return result.join("\n");
  }
}

export async function generateGenMod(writtenComponents: {
  [key: string]: string[];
}): Promise<void> {
  for (const component in writtenComponents) {
    const pathName = path.join(
      __dirname,
      "..",
      "..",
      "..",
      component,
      "src",
      "gen",
    );
    const absolutePathName = path.resolve(pathName);
    const code = ["// Auto-generated code!", "// No touchy!\n"];
    for (const typeName of writtenComponents[component]) {
      code.push(`pub mod ${snakeCase(typeName)};`);
    }

    await fs.promises.writeFile(
      path.join(absolutePathName, "mod.rs"),
      code.join("\n"),
    );
  }
}
