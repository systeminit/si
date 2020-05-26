/*!
 * Copyright 2019, OpenTelemetry Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

export class ApolloServer {}

import {
  GraphQLRequestContextWillSendResponse,
  ValueOrPromise,
} from "apollo-server-types";
import tapi, {
  Tracer,
  Span,
  SpanOptions,
  SpanKind,
  CanonicalCode,
  Status,
} from "@opentelemetry/api";

//import { infoPathToPath, getPrettyPath } from "./utils";
export type GraphQLPath = (string | number)[];

export function infoPathToPath(infoPath: any): GraphQLPath {
  return Object.keys(infoPath).reduce((prev: any, element: any) => {
    const next = infoPath[element];
    if (typeof next === "object" && !Array.isArray(next)) {
      return [...prev, ...infoPathToPath(next)];
    }
    return next !== undefined ? [...prev, next] : prev;
  }, []);
}

/** Turns GraphQL path to a human readable string. */
export function getPrettyPath(path: GraphQLPath): string {
  return "[" + path.join(", ") + "]";
}

const rootSpanKey = Symbol("root_span");

type ResolverTracingExtensionFactoryOpts = {
  tracer: Tracer;
};

type TracingContext = any & {
  [rootSpanKey]: Span;
};

/** Extension to report tracing data */
export function tracingExtensionFactory({
  tracer,
}: ResolverTracingExtensionFactoryOpts): () => PluginDefinition {
  return function resolverTracingExtension(): PluginDefinition {
    return {
      // Start graphql root span
      requestDidStart({ context }: { context: TracingContext }): object {
        // Start root span
        const options: SpanOptions = {
          kind: SpanKind.INTERNAL,
        };
        const parent = tracer.getCurrentSpan();
        if (parent !== null) {
          options.parent = parent;
        }
        const span = tracer.startSpan("apollo-server", options);
        return tracer.withSpan(span, function () {
          // Store root span on context
          context[rootSpanKey] = span;
          return {
            // End graphql root span
            willSendResponse(
              requestContext: GraphQLRequestContextWillSendResponse<any>,
            ): ValueOrPromise<void> {
              const span = context[rootSpanKey];
              if (requestContext.errors) {
                let status: Status | undefined;

                // Map status based on error extensions code.
                if (
                  requestContext.errors.find(
                    (err) =>
                      err.extensions &&
                      err.extensions.code === "INTERNAL_SERVER_ERROR",
                  )
                ) {
                  status = {
                    code: CanonicalCode.INTERNAL,
                  };
                } else if (
                  requestContext.errors.find(
                    (err) =>
                      err.extensions &&
                      err.extensions.code === "GRAPHQL_VALIDATION_FAILED",
                  )
                ) {
                  status = {
                    code: CanonicalCode.INVALID_ARGUMENT,
                  };
                } else {
                  status = {
                    code: CanonicalCode.UNKNOWN,
                  };
                }

                if (status) {
                  span.setStatus(status);
                }
              }
              console.log("ended root span");
              span.end();
            },
          };
        });
      },

      // Start and end resolver specific spans
      willResolveField(
        source,
        args,
        context,
        info,
      ): ((error: Error | null, result?: any) => void) | void {
        // Start resolver span
        const path = infoPathToPath(info.path);
        const spanName = getPrettyPath(path);
        const rootSpan = context[rootSpanKey];
        const span = tracer.startSpan(spanName, {
          parent: rootSpan,
          kind: SpanKind.INTERNAL,
        });
        console.log("created resolver span", { span });

        // End resolver span
        return (err): void => {
          if (err) {
            const status: Status = {
              code: CanonicalCode.INTERNAL,
              message: err.message,
            };
            span.setStatus(status);
            span.setAttribute("error.name", err.name);
            span.setAttribute("error.message", err.message);
            span.setAttribute("error.stack", err.stack);
          }
          console.log("ended resolver span", { span });

          span.end();
        };
      },
    };
  };
}
