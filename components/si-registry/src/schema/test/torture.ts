import {
  RegistryEntry,
  MenuCategory,
  ValidatorKind,
} from "../../registryEntry";

const service: RegistryEntry = {
  entityType: "torture",
  ui: {
    menuCategory: MenuCategory.Application,
    menuDisplayName: "torture test",
  },
  properties: [
    {
      type: "string",
      name: "standardString",
    },
    {
      type: "string",
      name: "validatedString",
      validation: [
        {
          kind: ValidatorKind.Alphanumeric,
        },
      ],
    },
    {
      type: "string",
      name: "allLayers",
    },
    {
      type: "array",
      name: "objectArray",
      itemProperty: {
        type: "object",
        properties: [
          {
            type: "string",
            name: "objectArrayString",
          },
        ],
      },
    },
    {
      type: "object",
      name: "level0",
      properties: [
        {
          type: "object",
          name: "level1",
          properties: [
            {
              type: "object",
              name: "level2",
              properties: [
                {
                  type: "string",
                  name: "level2String",
                },
              ],
            },
          ],
        },
      ],
    },
  ],
};

export default service;
