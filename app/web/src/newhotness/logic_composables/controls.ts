import { nextTick } from "vue";

// Generic handler for a tab action
export const handleTab = (e: KeyboardEvent, currentFocus?: HTMLElement) => {
  const focusable = Array.from(
    document.querySelectorAll('[tabindex="0"]'),
  ) as HTMLElement[];

  if (!currentFocus) return;
  const index = focusable.indexOf(currentFocus);

  if (e.shiftKey) {
    nextTick(() => {
      if (currentFocus && focusable) {
        if (index > 0) {
          focusable[index - 1]?.focus();
        } else {
          focusable[focusable.length - 1]?.focus();
        }
      }
    });
  } else if (index === focusable.length - 1) {
    // When you hit the last attribute, go back to the
    // fuzzy search instead of searching the document for more things to tab to.
    e.preventDefault();
    nextTick(() => {
      focusable[0]?.focus();
    });
  } else {
    nextTick(() => {
      focusable[index + 1]?.focus();
    });
  }
};
