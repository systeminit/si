import { ref, watch, inject, MaybeRefOrGetter, toValue, WatchSource, nextTick } from "vue";
import { useForm, formOptions } from "@tanstack/vue-form";
import { trace } from "@opentelemetry/api";
import { assertIsDefined, Context } from "../types";
import * as rainbow from "./rainbow_counter";

const tracer = trace.getTracer("si-vue");

/**
 * WHEN? You want to use this form when the data that displays initially
 * on the form is populated via the bifrost.
 *
 * WHY? form.reset(data) is a crucial aspect of this operation.
 * 1. TanStack forms are designed to be submitted just one time.
 * 2. So if you don't reset after a submission you can't re-submit.
 * 3. `defaultValues` is only set 1 time. Which means, if you were
 *    querying for the data for the form, the values will be blank
 *    because the form was created prior to having the data
 *
 * BONUS! you can `bifrosting` to indicate to the user that the form is
 * not capable of being re-submitted until its underlying data has
 * successfully mutated. Otherwise the UI experience can be:
 * 1. I submit a form
 * 2. I still see the previous value elsewhere in the UI
 * 3. I wonder... did my form submit work?!
 * 4. Once the patched data lands in the browser, you see a re-render
 * 5. Ah! It worked :)
 *
 * With `bifrosting` you can show a loader/spinner/disable a form
 * So the user know their form submission worked and we're waiting
 * for updated data
 */
export const useWatchedForm = <Data extends Record<string, string>>(label: string, resetBlank?: boolean) => {
  /**
   * Lifecycle of `bifrosting`
   *
   * false: prior to the form submission
   * TRUE: after the form submission
   * FALSE: after mutated data has been returned over the bifrost
   *        and is recomputed within `formData`
   *
   * You can pass `watchFn` if you want your `bifrosting` value
   * to be set by something other than the original formData.
   * This is useful when you have a blank create form with no data
   * And you want the spinner to stop when the data comes back out the
   * other side of the bifrost
   */
  const bifrosting = ref(false);

  const ctx = inject<Context>("CONTEXT");
  assertIsDefined(ctx);

  type ValidationFn = (props: { value: Data }) => (string | undefined) | { fields: Record<string, string | undefined> };
  type Validators = Partial<{
    onSubmit: ValidationFn;
    onChanged: ValidationFn;
    onBlur: ValidationFn;
    onSubmitAsync: ValidationFn;
    onChangedAsync: ValidationFn;
    onBlurAsync: ValidationFn;
  }>;

  // USE THIS RESET, NOT THE `form.reset()` FUNCTION!
  let dirty = false;
  /* eslint-disable-next-line @typescript-eslint/no-explicit-any */
  const reset = (f: any, data?: MaybeRefOrGetter<Data>) => {
    f.reset(data);
    dirty = false;
  };
  const newForm = ({
    data,
    onSubmit,
    watchFn,
    validators,
  }: {
    data: MaybeRefOrGetter<Data>;
    // NOTE: props also contains `formApi`, but I can't realistically type it here
    onSubmit: (props: { value: Data }) => Promise<void> | void;
    watchFn?: WatchSource<unknown>;
    validators?: Validators;
  }) => {
    const opts = formOptions({
      defaultValues: toValue(data),
    });

    // keep the form up to date with values until the user
    // changes form values, then we stop, and leave them be.
    watch(
      () => toValue(data),
      async (newData) => {
        if (!dirty) {
          // without this nextTick the form intermittently does not update!
          await nextTick();
          wForm.reset(newData);
        }
      },
    );
    const wForm = useForm({
      ...opts,
      listeners: {
        onChange(_props) {
          dirty = true;
        },
      },
      onSubmit: async (props) => {
        const start = performance.now();
        await tracer.startActiveSpan("watchedForm", async (span) => {
          span.setAttributes({
            workspaceId: ctx.workspacePk.value,
            changeSetId: ctx.changeSetId.value,
            "watched.label": label,
          });
          bifrosting.value = true;
          if (ctx.changeSetId.value !== ctx.headChangeSetId.value) rainbow.add(ctx.changeSetId.value, label);

          // Mark submission as complete and remove the rainbow spinner
          const markComplete = () => {
            bifrosting.value = false;
            dirty = false;
            rainbow.remove(ctx.changeSetId.value, label);
            wForm.reset(resetBlank ? undefined : props.value);
            if (span) {
              span.setAttribute("watched.duration", performance.now() - start);
              span.end();
            }
          };

          // Submit the form
          let hasSubmitted = false;
          try {
            await onSubmit(props);
            hasSubmitted = true;
          } catch (e) {
            // TODO report errors and display on caller forms
            // Cancel the spinner and bifrosting on failure
            markComplete();
          }

          // span.end() *must* get called in one of these permutations
          if (hasSubmitted) {
            if (watchFn) {
              watch(watchFn, markComplete, { deep: true });
            } else {
              watch(() => toValue(data), markComplete, { deep: true });
              // there are cases in which we don't have a watched value
              // or the system isn't responsive
              setTimeout(() => {
                rainbow.remove(ctx.changeSetId.value, label);
                span.setAttributes({
                  "watched.duration": performance.now() - start,
                  timed_out: true,
                });
                span.end();
              }, 1500);
            }
          }
        });
      },
      validators,
    });

    return wForm;
  };

  // PSA: don't use the TANSTACK `form.reset()` use our `wForm.reset(form)` fn
  // so we can keep track of the dirty state accurately...
  return { bifrosting, newForm, reset };
};

// NOTE: when the bifrost implements optimistic updates
// the time duration of `bifrosting` will be practically instant
// You still need this operation to reset the form!
