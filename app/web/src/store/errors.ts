import { StoreOnActionListenerContext, StateTree, _GettersTree, _ActionsTree } from "pinia";
import opentelemetry, { Span } from "@opentelemetry/api";

const handleStoreError = ({
  name, // name of the action
  store, // store instance, same as `someStore`
  args, // array of parameters passed to the action
  onError, // hook if the action throws or rejects
}: StoreOnActionListenerContext<string, StateTree, _GettersTree<StateTree>, _ActionsTree>) => {
  onError((error) => {
    const span = opentelemetry.trace.getActiveSpan();

    const _report = (span: Span) => {
      span.setAttribute("error.stacktrace", (error as Error)?.stack || "");
      span.setAttribute("error.message", (error as Error).message);
      span.setAttribute("error.store_$id", store.$id);
      span.setAttribute("error.store_fn_name", name);
      span.setAttribute("error.store_fn_args", JSON.stringify(args) || "");
    };

    if (span) {
      _report(span);
    } else {
      const tracer = opentelemetry.trace.getTracer("errorHandler");
      tracer.startActiveSpan("error", (span) => {
        _report(span);
        span.end();
      });
    }
  });
};

export default handleStoreError;
