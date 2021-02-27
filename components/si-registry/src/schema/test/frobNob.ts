import { PropObject } from "../../registryEntry";

const frobNob: PropObject = {
  name: "frobNob",
  type: "object",
  properties: [
    {
      type: "string",
      name: "chrisCornell",
    },
    {
      type: "object",
      name: "sugar",
      properties: [
        {
          type: "string",
          name: "patience",
        },
        {
          type: "object",
          name: "wait",
          properties: [{ type: "string", name: "slow" }],
        },
      ],
    },
  ],
};

export default frobNob;
