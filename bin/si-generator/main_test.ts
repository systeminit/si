import { assertEquals } from "https://deno.land/std@0.215.0/assert/mod.ts";
import { awsGenerate } from "./src/asset_generator.ts";
import { Prop } from "./src/props.ts";

Deno.test(function awsServiceProps() {
  const correctProps: Array<Prop> = [
    { kind: "string", name: "KeyName", variableName: "keyNameProp" },
    { kind: "boolean", name: "DryRun", variableName: "dryRunProp" },
    { kind: "string", name: "KeyType", variableName: "keyTypeProp" },
    {
      kind: "array",
      name: "TagSpecifications",
      variableName: "tagSpecificationsProp",
      entry: {
        kind: "object",
        name: "TagSpecificationsChild",
        variableName: "tagSpecificationsChildProp",
        children: [
          {
            kind: "string",
            name: "ResourceType",
            variableName: "resourceTypeProp",
          },
          {
            kind: "array",
            name: "Tags",
            variableName: "tagsProp",
            entry: {
              kind: "object",
              name: "TagsChild",
              variableName: "tagsChildProp",
              children: [
                { kind: "string", name: "Key", variableName: "keyProp" },
                {
                  kind: "string",
                  name: "Value",
                  variableName: "valueProp",
                },
              ],
            },
          },
        ],
      },
    },
    { kind: "string", name: "KeyFormat", variableName: "keyFormatProp" },
  ];
  const props = awsGenerate("ec2", "create-key-pair");
  assertEquals(props, correctProps);
});
