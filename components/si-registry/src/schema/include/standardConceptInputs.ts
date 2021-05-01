import { Input, Arity } from "../../registryEntry";

export const standardConceptInputs: Input[] = [
  {
    name: "application",
    types: ["application"],
    edgeKind: "component",
    arity: Arity.Many,
  },
];
