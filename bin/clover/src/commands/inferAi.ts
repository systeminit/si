import _logger from "../logger.ts";
import { allCfProps, loadCfDatabase } from "../cfDb.ts";
import type { CfProperty } from "../pipelines/types.ts";
import type { CfSchema } from "../pipelines/aws/schema.ts";
import OpenAI from "npm:openai@^4.104.0";
import _ from "lodash";
import { Inferred, loadInferred, saveInferred } from "../spec/inferred.ts";

const logger = _logger.ns("fetchSchema").seal();

const SAVE_INTERVAL = 5;
const GENERIC_PROMPT = `
You are a cloud engineer working with AWS CloudFormation. Given the CloudFormation documentation
for a resource property, you will decide whether the property has a known (limited) set of
possible values. If it does, you will respond with a list of those values in a JSON array of strings.
If it does not, you will respond with a JSON null value.

These values will be used to drive a UI to edit this property: if you respond with a JSON array,
the user will be presented with a dropdown list from which they can select only those values.
It would be very bad if the field actually allowed other values than the ones you specify,
because they would be unable to enter those values. So don't respond with an array unless you
are absolutely sure.

For example, given this CloudFormation documentation for the "Mode" property of AWS::ApiGateway::RestApi:

  This property applies only when you use OpenAPI to define your REST API. The \`\`Mode\`\` determines how API Gateway handles resource updates.
  Valid values are \`\`overwrite\`\` or \`\`merge\`\`.
  For \`\`overwrite\`\`, the new API definition replaces the existing one. The existing API identifier remains unchanged.
  For \`\`merge\`\`, the new API definition is merged with the existing API.
  If you don't specify this property, a default value is chosen. For REST APIs created before March 29, 2021, the default is \`\`overwrite\`\`. For REST APIs created after March 29, 2021, the new API definition takes precedence, but any container types such as endpoint configurations and binary media types are merged with the existing API.
  Use the default mode to define top-level \`\`RestApi\`\` properties in addition to using OpenAPI. Generally, it's preferred to use API Gateway's OpenAPI extensions to model these properties.

You would respond with:

  [ "overwrite", "merge" ]

You must respond *only* with a JSON array of strings, or a JSON null value, and no other text.
Do not include formatting or markdown delimiters. Do not include invalid JSON, and triple check
that you don't have extra quotes around your strings.
`;

export async function inferAi(options: {
  inferred: string;
  force?: boolean;
  services?: string[];
}) {
  const client = new OpenAI();
  const db = await loadCfDatabase(options);
  const inferred = await loadInferred(options.inferred);
  let lastSaved = Date.now();
  for (const cfSchema of _.sortBy(Object.values(db), "typeName")) {
    for (const { cfProp, cfPropPath } of allCfProps(cfSchema)) {
      // We only run inference on scalars
      if (!isScalarProperty(cfProp)) continue;

      // We don't run inference on read-only properties
      if (cfSchema.readOnlyProperties?.includes(`/properties${cfPropPath}`)) {
        logger.debug(
          `Ignoring ${cfSchema.typeName} ${cfPropPath} due to readOnlyProperties`,
        );
        continue;
      }

      // We can't infer without a description
      if (!cfProp.description) {
        logger.debug(
          `Ignoring ${cfSchema.typeName} ${cfPropPath} due to lack of description`,
        );
        continue;
      }

      // If we already have an inferred value for this description, skip it
      if (cfProp.description in inferred) continue;

      // Ask the AI for the possible values
      const specificPrompt = `
The CloudFormation documentation for the property ${cfPropPath} on ${cfSchema.typeName} is:

  ${cfProp.description}

Respond with the JSON array of strings representing the possible values for this property, or a JSON null value if it is not a limited set.
`;
      logger.debug(specificPrompt);
      const chatCompletion = await client.chat.completions.create({
        messages: [
          { role: "user", content: GENERIC_PROMPT },
          {
            role: "user",
            content: specificPrompt,
          },
        ],
        model: "gpt-4o",
      });

      // Check the resulting values are valid JSON array of strings (or null)
      let values: string[] | null;
      try {
        const parsed = JSON.parse(chatCompletion.choices[0].message.content!);
        if (
          parsed === null ||
          (Array.isArray(parsed) && parsed.every((v) => typeof v === "string"))
        ) {
          values = parsed;
        } else {
          throw new Error(
            `OpenAI response not string[] or null for property ${cfPropPath} on ${cfSchema.typeName}: ${parsed}`,
          );
        }
      } catch (e) {
        logger.error(chatCompletion);
        throw e;
      }

      logger.debug(chatCompletion.choices[0].message.content);

      inferred[cfProp.description] = { enum: values };
    }

    // Save periodically in case there are issues
    if (Date.now() - lastSaved > SAVE_INTERVAL) {
      saveInferred(options.inferred, inferred);
      lastSaved = Date.now();
    }
  }

  // Check that the results match any existing specs with enums
  for (const cfSchema of _.sortBy(Object.values(db), "typeName")) {
    validateEnums(cfSchema, inferred);
  }
}

type CfScalarProperty = Extract<
  CfProperty,
  { type: "integer" | "string" | "number" }
>;

function isScalarProperty(cfProp: CfProperty): cfProp is CfScalarProperty {
  return (
    typeof cfProp.type === "string" &&
    ["string", "number", "integer"].includes(cfProp.type)
  );
}

function validateEnums(cfSchema: CfSchema, inferred: Record<string, Inferred>) {
  for (const { cfProp, cfPropPath } of allCfProps(cfSchema)) {
    if (!cfProp.description) continue;
    const cfPropOverrides = inferred[cfProp.description];
    if (!cfPropOverrides) continue;
    if (!isScalarProperty(cfProp)) continue;

    // If there is no existing enum, there is no problem.
    if (cfProp.enum === undefined) {
      if (cfPropOverrides.enum) {
        // console.log(
        //   `NEW ${cfSchema.typeName} ${cfPropPath}: ${cfPropOverrides.enum}`,
        // );
      }
      continue;
    }

    // If there is an existing enum, and no inferred enum, there's no problem.
    if (cfPropOverrides.enum === null) continue;

    // Both have enums, so we need to compare them
    const realValues = new Set(cfProp.enum.map(String));
    const inferredValues = new Set(cfPropOverrides.enum);
    const missingValues = realValues.difference(inferredValues);
    if (missingValues.size > 0) {
      logger.warn(`
      ${cfSchema.typeName} ${cfPropPath}: Missing inferred enum values ${
        new Array(
          ...missingValues,
        ).toSorted()
      }:
      - Real enum:     ${cfProp.enum.map(String).toSorted()}
      - Inferred enum: ${cfPropOverrides.enum.toSorted()}

      ${cfProp.description}
      `);
      continue;
    }
    const extraValues = inferredValues.difference(realValues);
    if (extraValues.size > 0) {
      logger.debug(
        `ADD ${cfSchema.typeName} ${cfPropPath}: ${cfProp.enum} +${
          new Array(
            ...extraValues,
          ).toSorted()
        }`,
      );
    }
  }
}
