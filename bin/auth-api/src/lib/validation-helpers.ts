import Zod from 'zod';
import { ApiError } from "./api-error";

export function validate<Z extends Zod.Schema>(obj: any, schema: Z) {
  try {
    return schema.parse(obj) as Zod.infer<typeof schema>;
  } catch (err) {
    if (!(err instanceof Zod.ZodError)) throw err;

    const firstError = err.errors[0];
    const pathStr = firstError.path.join('.');

    throw new ApiError('BadRequest', 'ValidationError', `Invalid \`${pathStr}\` - ${firstError.message}`);
  }
}

export const ALLOWED_INPUT_REGEX = /^[a-zA-Z0-9-.,_@/+ ]*$/;

export const ALLOWED_URL_REGEX = "^https?://([\\da-z.-]+)(:\\d+)?(/[\\w .-]*)*/?$";
