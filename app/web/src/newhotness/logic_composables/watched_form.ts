import { ref, unref, watch, Ref, ComputedRef } from "vue";
import { useForm, formOptions } from "@tanstack/vue-form";

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
export const useWatchedForm = <Data>() => {
  /**
   * Lifecycle of `bifrosting`
   *
   * false: prior to the form submission
   * TRUE: after the form submission
   * FALSE: after mutated data has been returned over the bifrost
   *        and is recomputed within `formData`
   */
  const bifrosting = ref(false);

  const newForm = (
    formData: Ref<Data> | ComputedRef<Data>,
    // NOTE: props also contains `formApi`, but I can't realistically type it here
    onSubmit: (props: { value: Data }) => void,
  ) => {
    const opts = formOptions({
      defaultValues: unref(formData),
    });
    const wForm = useForm({
      ...opts,
      onSubmit: (props) => {
        onSubmit(props);
        bifrosting.value = true;
      },
    });

    watch(formData, () => {
      bifrosting.value = false;
      wForm.reset(unref(formData));
    });

    return wForm;
  };

  return { bifrosting, newForm };
};

// NOTE: when the bifrost implements optimistic updates
// the time duration of `bifrosting` will be practically instant
// You still need this operation to reset the form!
