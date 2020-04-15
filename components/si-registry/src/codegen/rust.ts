import { Component } from "@/component";

import { snakeCase } from "change-case";
import ejs from "ejs";
import fs from "fs";
import path from "path";

export class CodegenRust {
  component: Component;

  constructor(component: Component) {
    this.component = component;
  }

  async writeCode(part: string, code: string): Promise<void> {
    const createdPath = await this.makePath();
    await fs.promises.writeFile(
      path.join(createdPath, `${snakeCase(part)}.rs`),
      code,
    );
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
      code.push(`mod ${snakeCase(typeName)};`);
    }

    await fs.promises.writeFile(
      path.join(absolutePathName, "mod.rs"),
      code.join("\n"),
    );
  }
}
