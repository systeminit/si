/* eslint-disable consistent-return,no-console */
import _ from "lodash";
import Debug from "debug";
import PrettyError from "pretty-error";
import chalk from "chalk";

import { ApiError } from "./api-error";
// import { getLogDnaLogger } from '../external-services/logdna';
// import { logErrorToSentry } from '../external-services/sentry';

// const logDnaLogger = getLogDnaLogger();

// // pass through errors with the logger itself to sentry
// if (logDnaLogger) {
//   logDnaLogger.on('error', (err) => {
//     console.error('ERROR WITH LOGGER', err);
//     logErrorToSentry(null, null, err);
//   });
// }

// add some fancy formatting to errors in the console
// and skip some packages/lines that aren't helpful
// TODO: add more formatting rules? make prettier?
const prettyError = new PrettyError();
prettyError.skipNodeFiles();
prettyError.skipPackage(
  "koa",
  "@koa",
  "@koa/router",
  "koa-compose",
  "koa-body",
);

// used to toggle swallowing 500 errors in test mode - only when testing 500 handling on purpose...
let swallowUnexpectedErrors = false;
export function setSwallowErrors(newValue: boolean) {
  swallowUnexpectedErrors = newValue;
}

const debuggers: Record<string, Debug.Debugger> = {};

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

  // log to console using debug with the topic set by meta.type
  const debugTopic = meta.type;
  if (!debuggers[debugTopic]) debuggers[debugTopic] = Debug(debugTopic);
  const debugLogger = debuggers[debugTopic];
  debugLogger(message);
  const metaWithoutType = _.omit(meta, "type");
  if (!_.isEmpty(metaWithoutType)) debugLogger(metaWithoutType); // maybe dont always want to show?

  // log to LogDNA
  // if (logDnaLogger) {
  //   logDnaLogger.log(message, {
  //     level: (error ? "error" : "info") as any,
  //     meta,
  //   });
  // }

  // only unexpected errors get passed through to sentry
  if (
    error
    && error instanceof Error
    && !(error as ApiError).expectedError
    && !swallowUnexpectedErrors
  ) {
    console.log(chalk.red("------ CAUGHT EXCEPTION ------"));
    console.log(prettyError.render(error));
    console.log(chalk.red("------------------------------"));

    // logErrorToSentry(_.pick(meta, "url", "method"), meta.user, error);
  }
}

// async function flush() {
//   // TODO: probably want to flush logs before shutdown
// }
