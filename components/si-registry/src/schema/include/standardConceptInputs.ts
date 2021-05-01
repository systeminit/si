import { Input, Arity } from "../../registryEntry";

export const standardConceptInputs: Input[] = [
  {
    name: "application",
    types: ["application"],
    edgeKind: "component",
    arity: Arity.Many,
  },
  {
    name: "poop",
    types: ["service"],
    edgeKind: "deployment",
    arity: Arity.Many,
  },
  {
    name: "implementations",
    types: "implementations",
    edgeKind: "configures",
    arity: Arity.Many,
  },
];
