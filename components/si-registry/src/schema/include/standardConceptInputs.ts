import { Input, Arity } from "../../registryEntry";

export const standardConceptInputs: Input[] = [
  {
    name: "implementations",
    types: "implementations",
    edgeKind: "configures",
    arity: Arity.Many,
  },
  {
    name: "dependencies",
    types: "dependencies",
    edgeKind: "configures",
    arity: Arity.Many,
  },
];
