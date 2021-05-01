import {
  RegistryEntry,
  MenuCategory,
  ValidatorKind,
  SchematicKind,
  NodeKind,
} from "../../registryEntry";

const torture: RegistryEntry = {
  entityType: "torture",
  nodeKind: NodeKind.Concrete,
  ui: {
    menuCategory: MenuCategory.Application,
    menuDisplayName: "torture test",
    schematicKinds: [SchematicKind.Component, SchematicKind.Deployment],
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
      type: "boolean",
      name: "boolMeBaby",
    },
    {
      type: "string",
      name: "supers3cret",
      widget: {
        name: "password",
      },
    },
    {
      type: "string",
      name: "biggerText",
      widget: {
        name: "textArea",
      },
    },
    {
      type: "number",
      name: "gimme a number",
      validation: [
        {
          kind: ValidatorKind.Int,
        },
      ],
    },
    {
      type: "string",
      name: "selectable",
      widget: {
        name: "select",
        options: {
          items: [
            { label: "first", value: "first" },
            { label: "second", value: "second" },
            { label: "third", value: "third" },
          ],
        },
      },
    },
    {
      type: "map",
      name: "mappy",
      valueProperty: {
        type: "string",
      },
    },
    {
      type: "array",
      name: "stringArray",
      itemProperty: {
        type: "string",
      },
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
      type: "array",
      name: "complexArray",
      itemProperty: {
        type: "object",
        properties: [
          {
            type: "string",
            name: "only",
          },
          {
            type: "object",
            name: "what",
            properties: [
              {
                type: "array",
                name: "foolish",
                itemProperty: {
                  type: "object",
                  properties: [
                    {
                      type: "number",
                      name: "validatedNumber",
                      validation: [
                        {
                          kind: ValidatorKind.Int,
                          options: {
                            max: 10,
                            min: 2,
                          },
                        },
                      ],
                    },
                    {
                      type: "map",
                      name: "imagine",
                      valueProperty: {
                        type: "string",
                      },
                    },
                  ],
                },
              },
            ],
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
  commands: [
    {
      name: "doit5.pl",
      description: "do what I want",
      args: [
        {
          type: "string",
          name: "echo",
        },
      ],
    },
    {
      name: "universal:deploy",
      description: "deploy this entity",
      aliasTo: "doit5.pl",
    },
  ],
  // RulesetGroup --> Set of (action, ruleset) (select between them)
  // RulesetGroup is tied to entity type
  // you can set a default ruleset group for the entity type
  // you can set a default ruleset group for the entity type and system
  // you can set a specific ruleset gfroup for a given entity instance
  // you can set a specific ruleset group for a given entity instance and system
  //
  // Each ruleset group contains
  //   Action: Ruleset [
  //     boolean expression -> workflow <-- rules!
  //     boolean expression -> workflow
  //   ]
  //
  // Rulesets can be set to a value for the rulesetgroup
  // Rulesets can be set to a value for the rulesetgroup and entity instance
  // Rulesets can be set to a value for the rulesetgroup and entity instance system
  //
  actions: [
    {
      name: "universal:deploy",
    },
  ],
};

export default torture;
