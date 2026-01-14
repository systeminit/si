import FontFaceObserver from "fontfaceobserver";
import { computed, ComputedRef, inject, InjectionKey, provide, ref } from "vue";

const FONTS_LOADED_INJECTION_KEY: InjectionKey<ComputedRef<boolean>> = Symbol("FONTS_LOADED");

// TODO: pull specific font from tailwind or body's css
// TODO: support multiple fonts?
const fontFaceObserver = new FontFaceObserver("Inter");
const customFontsLoaded = ref(false);

// if some crazy devops nerd is blocking remote fonts (like Paulo) then this never fires so we'll need to make sure we don't crash in this scenario
// for now we'll just wait until the timeout and pretend we loaded the fonts, but we probably want to do something smarter
fontFaceObserver
  .load(null, 5000)
  .then(() => {
    customFontsLoaded.value = true;
  })
  .catch((_err) => {
    // TODO: something smarter. Maybe we expose multiple states - not-loaded, loaded, blocked
    customFontsLoaded.value = true;
  });

// sets up provider - to be used in root level App.vue component
export function useCustomFontsLoadedProvider() {
  provide(
    FONTS_LOADED_INJECTION_KEY,
    computed(() => customFontsLoaded.value),
  );
}

export function useCustomFontsLoaded() {
  return inject(
    FONTS_LOADED_INJECTION_KEY,
    // default value - will never actually be used but helps TS not complain
    computed(() => false),
  );
}
