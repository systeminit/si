import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { SchemasApi, FuncsApi, SchemaVariantFunc } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";

const toolName = "schema-create-edit-get";
const title = "Create, edit, or get information about a schema";
const description = `
<description>
Create a new schema, edit an existing schema, or get information about an existing schema.
</description>
<usage>
If the user does not specify a change set, ask the user which change set to use or if a new one should be created. If the user gives a schema name to edit or get information about, use the schema-find tool to find the corresponding schemaId. If the user is trying to add code to an existing asset function for a schema, get the current code for that schema and add to it. Before writing any asset function code, reference the documentation to get the syntax right. Always put each prop within an asset function in a const before adding it to the asset. Do not mention schema variants or locking/unlocking to the user. This tool cannot be used on the HEAD change set.
</usage>
`;

const DEFAULT_SCHEMA_DEFINITION_FUNCTION = `function main() {
    const asset = new AssetBuilder();
    return asset.build();
}`;

const schemaCreateEditGetInputSchemaRaw = {
  changeSetId: z
    .string()
    .describe(
      "The change set to create, edit, or get a schema in; schemas cannot be manipulated on HEAD.",
    ),
  name: z.string().min(1).optional().describe("The name of the schema. A name is required for creating a new schema."),
  description: z.string().optional().describe("The description of the schema."),
  schemaId: z.string().optional().describe("The id of the schema you want to edit. If none is given, create a new schema. If only a change set id and schema id are given, just get information about the schema."),
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
      <aws-good-example>
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

        const extraProp = new PropBuilder()
            .setName("extra")
            .setKind("object")
            .setHidden(false)
            .setWidget(new PropWidgetDefinitionBuilder()
                .setKind("header")
                .build())
            .addChild(
                new PropBuilder()
                .setName("Region")
                .setKind("string")
                .setHidden(false)
                .setWidget(new PropWidgetDefinitionBuilder()
                    .setKind("text")
                    .build())
                .suggestSource({
                    schema: "Region",
                    prop: "/domain/region"
                })
                .build()
            )
            .build();

        const AWSCredentialSecretProp = new SecretPropBuilder()
            .setName("AWS Credential")
            .setSecretKind("AWS Credential")
            .build();

        asset.addProp(stringProp);
        asset.addProp(codeEditorProp);
        asset.addProp(objectProp);
        asset.addProp(arrayProp);
        asset.addProp(mapProp);
        asset.addProp(extraProp);
        asset.addSecretProp(AWSCredentialSecretProp)

        return asset.build();
      </aws-good-example>
      `,
    ),
};

const schemaCreateEditGetOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z.object({
    schemaId: z.string().describe("the schema id"),
    name: z.string().describe("the schema name"),
    definitionFunction: z.string().describe("the schema definition function"),
    functions: z.array(z.object({
      id: z.string().describe("the function id"),
      funcKind: z.union([
        z.object({
          actionKind: z.string().describe("the action kind"),
          kind: z.literal("action").describe("the function kind"),
        }),
        z.object({
          managementFuncKind: z.string().describe("the management function kind"),
          kind: z.literal("management").describe("the function kind"),
        }),
        z.object({
          funcKind: z.string().describe("the function kind"),
          kind: z.literal("other"),
        }),
      ]),
    })).describe("the functions attached to the schema"),
  }),
};
const schemaCreateEditGetOutputSchema = z.object(schemaCreateEditGetOutputSchemaRaw);
type SchemaCreateEditGetOutputData = z.infer<typeof schemaCreateEditGetOutputSchema>["data"];

export function schemaCreateEditGetTool(server: McpServer) {
  server.registerTool(
    toolName,
    {
      title,
      description: generateDescription(
        description,
        "schemaCreateEditGet",
        schemaCreateEditGetOutputSchema,
      ),
      inputSchema: schemaCreateEditGetInputSchemaRaw,
      outputSchema: schemaCreateEditGetOutputSchemaRaw,
    },
    async ({ changeSetId, definitionFunction, schemaId, ...createOrEditSchemaV1Request }) => {
      return await withAnalytics(toolName, async () => {
        const siSchemasApi = new SchemasApi(apiConfig);
        const siFuncsApi = new FuncsApi(apiConfig);

        let hints, touchedSchemaId, touchedDefinitionFunction, touchedName: string, touchedFunctions;

        try {
          if (schemaId) {
            // edit an existing schema or get information about it

            // first we need to make sure we have an unlocked schema variant
            const responseUnlock = await siSchemasApi.unlockSchema({
              workspaceId: WORKSPACE_ID,
              changeSetId,
              schemaId,
            });

            const schemaVariantId = responseUnlock.data.unlockedVariantId;

            // then we need to get the info about that unlocked variant
            const responseGetVariant = await siSchemasApi.getVariant({
              workspaceId: WORKSPACE_ID,
              changeSetId,
              schemaId,
              schemaVariantId,
            });

            const assetFuncId = responseGetVariant.data.assetFuncId;

            // then we need to get the current asset func code
            const responseGetFunc = await siFuncsApi.getFunc({
              workspaceId: WORKSPACE_ID,
              changeSetId,
              funcId: assetFuncId,
            });

            // populate data to return from the tool
            touchedSchemaId = schemaId;
            touchedDefinitionFunction = definitionFunction ?? responseGetFunc.data.code;
            touchedName = createOrEditSchemaV1Request.name ?? responseGetVariant.data.displayName;
            touchedFunctions = responseGetVariant.data.variantFuncs;

            // information gathering complete, now only move onto updating if we have new data
            if (definitionFunction || Object.values(createOrEditSchemaV1Request).some(value => value != undefined)) {
              // if this schema is a builtin, we need to warn the user accordingly
              if (responseGetVariant.data.installedFromUpstream) {
                hints = "Warn the user that because this schema was created by System Initiative that they will lose their customizations to it if they upgrade the schema. Repeat this warning every time the user edits any builtin schema.";
              }

              // so that we can build the final request with the current data as the default
              const updateSchemaVariantV1Request = {
                name: responseGetVariant.data.displayName,
                description: responseGetVariant.data.description,
                category: responseGetVariant.data.category,
                color: responseGetVariant.data.color,
                link: responseGetVariant.data.link,
                code: responseGetFunc.data.code,
                // then injecting our new data to overwrite any field we put a value for
                ...createOrEditSchemaV1Request,
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
            }
          } else {
            // create a new schema

            // a new schema must have a name
            const name = createOrEditSchemaV1Request.name;
            if (!name) {
              return errorResponse({
                message: "A name is required to make a new schema.",
                hints: "Ask the user to give this new schema a name."
              });
            }

            const code = definitionFunction ?? DEFAULT_SCHEMA_DEFINITION_FUNCTION;

            // next we call the endpoint to create a new schema
            const responseCreate = await siSchemasApi.createSchema({
              workspaceId: WORKSPACE_ID,
              changeSetId: changeSetId,
              createSchemaV1Request: {
                ...createOrEditSchemaV1Request,
                name,
                code,
              },
            });
            // populate data to return from the tool
            touchedSchemaId = responseCreate.data.schemaId;
            touchedDefinitionFunction = code;
            touchedName = createOrEditSchemaV1Request.name as string;
            touchedFunctions = [] as SchemaVariantFunc[];
          }
          const data: SchemaCreateEditGetOutputData = {
            schemaId: touchedSchemaId,
            definitionFunction: touchedDefinitionFunction,
            name: touchedName,
            functions: touchedFunctions,
          };
          return successResponse(data, hints);
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
