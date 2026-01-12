/* eslint-disable consistent-return,no-console */
import _ from "lodash";

import { ApiError } from "./api-error";

// used to toggle swallowing 500 errors in test mode - only when testing 500 handling on purpose...
let swallowUnexpectedErrors = false;
export function setSwallowErrors(newValue: boolean) {
  swallowUnexpectedErrors = newValue;
}

// general logger function that writes a log to console (using debug), logdna, and sentry (if an error)
// note this should not be used directly in http routes or queues, instead using ctx.log which includes extra metadata
export function log(
  message: string,
  meta: {
    type?: string;
    message?: string;
    statusCode?: number;
    error?: Error | ApiError | any;
    [key: string]: any;
  } = {},
) {
  if (!_.isString(message)) throw new Error("Missing log message");
  if (meta === null) meta = {};
  if (meta.id) throw new Error("Do not set ID in data to log!");

  const error = meta.error;

  if (error) {
    if (meta.message) meta.message += ` - ${error.name}`;
    // replace the raw error with something better for logging
    if (error) {
      meta.error = _.pickBy(
        _.pick(error, [
          // extra _.pickBy removes empty keys (details)
          "name",
          "generalType",
          "message",
          "details",
          "stack",
        ]),
      );

      // if expected error, trim off stack
      if (meta.statusCode && meta.statusCode < 500) {
        meta.error = _.omit(meta.error, ["stack"]);
      }
    }

    // if the error looks like an http request, we add it so it gets captured
    if (error.response) {
      meta.error.httpResponse = JSON.stringify(error.response.data);
    }
  }

  // TODO: check meta for data that is too deep or circular. It should be kept simple!

  // we use meta.type both to categorize logs on logdna and also as a debug "topic"
  // meta.type defaults to "general" if nothing is set
  meta.type = meta.type || "general";

  // For unexpected errors (500+), mark them for easier filtering
  const isUnexpectedError = error
    && error instanceof Error
    && !(error as ApiError).expectedError
    && !swallowUnexpectedErrors;

  console.log(JSON.stringify({
    timestamp: new Date().toISOString(),
    level: meta.error ? "error" : "info",
    message,
    ...meta,
    ...(isUnexpectedError && { unexpected: true }),
  }));
}
