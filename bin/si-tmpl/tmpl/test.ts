import { TemplateContext } from "../src/template.ts";

export default function (c: TemplateContext) {
  c.name("van morrison");
  c.changeSet(`${c.name()} always uses the ${c.invocationKey()}`);
  c.search([
    "schema:*"
  ]);
}
