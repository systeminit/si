import {
  RegistryEntry,
  MenuCategory,
  ValidatorKind,
} from "../../registryEntry";

const torture: RegistryEntry = {
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
