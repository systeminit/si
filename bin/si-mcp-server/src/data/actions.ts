import { z } from "zod";

export const ActionSchemaRaw = {
  actionId: z.string().describe("the action id"),
  componentId: z.string().describe("the component id the action is on"),
  componentName: z.string().describe("the component name the action is on"),
  funcRunId: z.string().nullable().optional().describe(
    "the function run id for the last executed function for this action",
  ),
  schemaName: z.string().describe(
    "the schema name of the component the action is on",
  ),
  name: z.string().describe("The name of the Action"),
  kind: z.string().describe("the kind of action"),
  state: z.enum(["Dispatched", "Failed", "OnHold", "Queued", "Running"])
    .describe(
      "the current state of the action. 'Dispatched' means the action is eligible to run, and has been sent to the job queue for execution. 'Failed' means the action failed during execution. 'OnHold' means that the action is queued to run, but has been manually paused this action won't run, nor will any dependent actions, until the action is put back into the Queued state. 'Queued' means the action is elgible to run once all of its prerequisite actions have succeeded. 'Running' means the action has been dispatched, and has the job is being executed.",
    ),
};
export const ActionSchema = z.object(ActionSchemaRaw);

export type ActionList = z.infer<typeof ActionSchema>;
