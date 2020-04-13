import { Component } from "@/component";
import ejs from "ejs";

export class CodegenProtobuf {
  component: Component;

  constructor(component: Component) {
    this.component = component;
  }

  generateString(): string {
    return ejs.render(
      "<%- include('protobuf/full', { component: component }) %>",
      {
        component: this.component,
      },
      {
        filename: __filename,
      },
    );
  }
}
