import {
  RegistryEntry,
  MenuCategory,
  ValidatorKind,
} from "../../registryEntry";
import frobNob from "./frobNob";

const leftHandPath: RegistryEntry = {
  entityType: "leftHandPath",
  ui: {
    menuCategory: MenuCategory.Kubernetes,
    menuDisplayName: "leftHandPath",
  },
  properties: [
    {
      type: "string",
      name: "simpleString",
      validation: [
        {
          kind: ValidatorKind.Alphanumeric,
        },
      ],
    },
    { type: "number", name: "simpleNumber" },
    {
      type: "array",
      name: "wanda",
      itemProperty: {
        type: "string",
        validation: [
          {
            kind: ValidatorKind.Alphanumeric,
          },
        ],
      },
    },
    {
      type: "array",
      name: "abnormal",
      itemProperty: {
        type: "object",
        properties: [{ type: "string", name: "illusion" }],
      },
    },
    {
      type: "object",
      name: "party",
      properties: [
        {
          type: "string",
          name: "poop",
        },
      ],
    },
    {
      type: "object",
      name: "nestedArrays",
      properties: [
        {
          type: "array",
          name: "darkness",
          itemProperty: {
            type: "object",
            properties: [
              {
                type: "object",
                name: "surrounds",
                properties: [
                  {
                    type: "array",
                    name: "justice",
                    itemProperty: {
                      type: "object",
                      properties: [
                        {
                          type: "string",
                          name: "prevails",
                        },
                      ],
                    },
                  },
                ],
              },
            ],
          },
        },
      ],
    },
    frobNob,
  ],
};

export default leftHandPath;
