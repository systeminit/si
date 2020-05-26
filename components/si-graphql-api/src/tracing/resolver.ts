import * as traceApi from "@opentelemetry/api";
import { camelCase } from "change-case";

interface ResolverSpanOptions {
  resolverType?: string;
  mutation?: boolean;
  root?: boolean;
  context: any;
}

export function resolverSpan(
  name: string,
  options: ResolverSpanOptions = { context: {} },
): traceApi.Span {
  const resolverType = options.resolverType || "resolver";
  const trace = traceApi.trace.getTracer("si-graphql-api");
  const span = trace.startSpan(`graphql.${resolverType} ${camelCase(name)}`);
  span.setAttribute("graphql.resolver", true);
  span.setAttribute("graphql.root", options.root);
  if (options.mutation) {
    span.setAttribute("graphql.mutation", true);
  } else {
    span.setAttribute("graphql.query", true);
  }
  if (options.context?.user) {
    span.setAttribute("user_id", options.context.user.userId);
    span.setAttribute(
      "billing_account_id",
      options.context.user.billingAccountId,
    );
    span.setAttribute("authenticated", true);
  } else {
    span.setAttribute("authenticated", false);
  }
  return span;
}
