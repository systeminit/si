import { z } from "zod";

export const BaseSchema = z.object({
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  deepLink: z
    .string()
    .optional()
    .describe(
      "Providing the user this link will make it seamless for them to review what has been created within the System Initiative web application. You can tell them that this is the direct link to the resource.",
    ),
});

export type BaseSchemaType = z.infer<typeof BaseSchema>;

export function createResponseSchema<T extends z.ZodRawShape>(dataSchema: T) {
  return BaseSchema.extend({
    data: z.object(dataSchema).optional(),
  });
}

export function createRequiredDataResponseSchema<T extends z.ZodRawShape>(dataSchema: T) {
  return BaseSchema.extend({
    data: z.object(dataSchema),
  });
}
