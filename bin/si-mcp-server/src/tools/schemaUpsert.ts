import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { SchemasApi, FuncsApi } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";

const name = "schema-upsert";
const title = "Create or edit a schema";
const description = `
<description>
Create a new schema or edit an existing schema.
</description>
<usage>
If the user does not specify a change set, ask the user which change set to use or if a new one should be created. If the user gives a schema name to edit, use the schema-find tool to find the corresponding schemaId. If the user is trying to add code to an existing asset function for a schema, get the current code for that schema and add to it. Before writing any asset function code, reference the documentation to get the syntax right. Always put each prop within an asset function in a const before adding it to the asset. Do not mention schema variants or locking/unlocking to the user.
</usage>
`;

const DEFAULT_SCHEMA_DEFINITION_FUNCTION = `function main() {
    const asset = new AssetBuilder();
    return asset.build();
}`;

const SchemaUpsertInputSchemaRaw = {
  changeSetId: z
    .string()
    .describe(
      "The change set to create or edit a schema in; schemas cannot be created or edited on HEAD.",
    ),
  name: z.string().min(1).optional().describe("The name of the schema. A name is required for creating a new schema."),
  description: z.string().optional().describe("The description of the schema."),
  schemaId: z.string().optional().describe("The id of the schema you want to edit. If none is given, create a new schema."),
  link: z
    .string()
    .optional()
    .describe("A link to documentation about the thing the schema represents."),
  category: z.string().optional().describe("The category of the schema"),
  color: z.string().optional().describe("The schema's color. Must be a hex color, convert any color words into a hex value."),
  definitionFunction: z
    .string()
    .optional()
    .describe(
      `
      <description>A typescript function, starting with "function main() {", defining the schema's properties using an AssetBuilder. Documentation on how to write this function can be found at https://docs.systeminit.com/reference/asset/schema</description>
      <good-example>
        const asset = new AssetBuilder();

        const stringProp = new PropBuilder()
          .setName("StringProp")
          .setKind("string")
          .setDocumentation("Documentation text explaining this prop goes here.")
          .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
          .build();
        
        const integerProp = new PropBuilder()
          .setName("IntegerProp")
          .setKind("integer")
          .setDocumentation("Documentation text explaining this prop goes here.")
          .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
          .build();
        
        const booleanProp = new PropBuilder()
          .setName("BooleanProp")
          .setKind("boolean")
          .setDocumentation("Documentation text explaining this prop goes here.")
          .setWidget(new PropWidgetDefinitionBuilder().setKind("checkbox").build())
          .build();
        
        const codeEditorProp = new PropBuilder()
          .setName("CodeEditorProp")
          .setKind("string")
          .setDocumentation("Documentation text explaining this prop goes here.")
          .setWidget(new PropWidgetDefinitionBuilder().setKind("codeEditor").build())
          .build();

        const passwordProp = new PropBuilder()
          .setName("PasswordProp")
          .setKind("string")
          .setDocumentation("Documentation text explaining this prop goes here.")
          .setWidget(new PropWidgetDefinitionBuilder().setKind("password").build())
          .build();

        const objectProp = new PropBuilder()
          .setName("ObjectProp")
          .setKind("object")
          .addChild(booleanProp)
          .build();
        
        const arrayProp = new PropBuilder()
          .setName("ArrayProp")
          .setKind("array")
          .setEntry(passwordProp)
          .build();

        const mapProp = new PropBuilder()
          .setName("MapProp")
          .setKind("map")
          .setEntry(integerProp)
          .build();

        asset.addProp(stringProp);
        asset.addProp(codeEditorProp);
        asset.addProp(objectProp);
        asset.addProp(arrayProp);
        asset.addProp(mapProp);

        return asset.build();
      </good-example>
      `,
    ),
};

const SchemaUpsertOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z.object({
    schemaId: z.string().describe("the schema id"),
  }),
};
const SchemaUpsertOutputSchema = z.object(SchemaUpsertOutputSchemaRaw);
type SchemaUpsertOutputData = z.infer<typeof SchemaUpsertOutputSchema>["data"];

export function schemaUpsertTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "schemaUpsert",
        SchemaUpsertOutputSchema,
      ),
      inputSchema: SchemaUpsertInputSchemaRaw,
      outputSchema: SchemaUpsertOutputSchemaRaw,
    },
    async ({ changeSetId, definitionFunction, schemaId, ...upsertSchemaV1Request }) => {
      return await withAnalytics(name, async () => {
        const siSchemasApi = new SchemasApi(apiConfig);
        const siFuncsApi = new FuncsApi(apiConfig);

        let touchedSchemaId;

        try {
          if (schemaId) {
            // edit an existing schema

            // first we need to make sure we have an unlocked schema variant
            const response = await siSchemasApi.unlockSchema({
              workspaceId: WORKSPACE_ID,
              changeSetId,
              schemaId,
            });

            const schemaVariantId = response.data.unlockedVariantId;

            // then we need to get the info about that unlocked variant
            const response2 = await siSchemasApi.getVariant({
              workspaceId: WORKSPACE_ID,
              changeSetId,
              schemaId,
              schemaVariantId,
            });

            const assetFuncId = response2.data.assetFuncId;

            // then we need to get the current asset func code
            const response3 = await siFuncsApi.getFunc({
              workspaceId: WORKSPACE_ID,
              changeSetId,
              funcId: assetFuncId,
            });

            // so that we can build the final request with the current data as the default
            const updateSchemaVariantV1Request = {
              name: response2.data.displayName,
              description: response2.data.description,
              category: response2.data.category,
              color: response2.data.color,
              link: response2.data.link,
              code: response3.data.code,
              // then injecting our new data to overwrite any field we put a value for
              ...upsertSchemaV1Request,
            };

            // then if we gave new asset function code, overwrite the old code here
            if (definitionFunction) {
              updateSchemaVariantV1Request.code = definitionFunction;
            }

            // and finally we actually call the endpoint to edit the unlocked schema variant!
            await siSchemasApi.updateSchemaVariant({
              workspaceId: WORKSPACE_ID,
              changeSetId,
              schemaId,
              schemaVariantId,
              updateSchemaVariantV1Request,
            });

            touchedSchemaId = schemaId;
          } else {
            // create a new schema

            // a new schema must have a name
            if (!upsertSchemaV1Request.name) {
              return errorResponse({
                message: "A name is required to make a new schema.",
                hints: "Ask the user to give this new schema a name."
              });
            }

            // then we call the endpoint to create a new schema
            await siSchemasApi.createSchema({
              workspaceId: WORKSPACE_ID,
              changeSetId: changeSetId,
              createSchemaV1Request: {
                ...upsertSchemaV1Request,
                code: definitionFunction ?? DEFAULT_SCHEMA_DEFINITION_FUNCTION,
              },
            });

            // after creating the new schema, we need to get its id to return to Claude
            const response = await siSchemasApi.findSchema({
              workspaceId: WORKSPACE_ID,
              changeSetId: changeSetId!,
              schema: upsertSchemaV1Request.name,
            });
            touchedSchemaId = response.data.schemaId;
          }
          const data: SchemaUpsertOutputData = {
            schemaId: touchedSchemaId,
          };
          return successResponse(data);
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
