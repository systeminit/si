import { z } from "zod";

export const ChangeSetSchemaRaw = {
  id: z.string().describe("Change Set ID"),
  isHead: z.boolean().describe("True if the change set is HEAD; false if not"),
  name: z.string().describe("The name of the Change Set"),
  status: z.enum([
    "Abandoned",
    "Applied",
    "Approved",
    "Failed",
    "NeedsApproval",
    "Open",
    "Rejected",
  ]).describe(
    "The status of the change set. 'Abandoned' means it is no longer accessible. 'Applied' means it has been applied to HEAD. 'Approved' means any neccessary approvals have been applied. 'Failed' means a snapshot migrations has failed. 'NeedsApproval' means applying to HEAD is desired, but approvals are required first. 'Open' means it is available for users to modify. 'Rejected' means a request to apply with approval was rejected.",
  ),
};
export const ChangeSetSchema = z.object(ChangeSetSchemaRaw);

export type ChangeSet = z.infer<typeof ChangeSetSchema>;
