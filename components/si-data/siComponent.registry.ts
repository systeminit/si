import { registry } from "../si-registry/src/componentRegistry";
import { PropEnum } from "../si-registry/src/prop/enum";
import { Component } from "../si-registry/src/component";

registry.component({
  typeName: "data",
  displayTypeName: "SI Data",
  options(c: Component) {
    c.internalOnly.addEnum({
      name: "queryComparison",
      label: "Query Comparison",
      option(p: PropEnum) {
        p.options = ["equals", "notEquals", "contains", "like", "notLike"];
      },
    });
  },
});
