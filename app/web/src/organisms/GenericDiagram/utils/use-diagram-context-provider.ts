// playing with using provide/inject to pass down some context about the stage
// for example zoom level, visible viewport boundaries, etc
// this will let us access those things in any components within the diagram
// without having to pass them down through many levels...

import { computed, ComputedRef, inject, InjectionKey, provide, Ref } from "vue";

// TODO: maybe use a single key to inject a diagram context object rather than individual bits of info?
const DIAGRAM_ZOOM_INJECTION_KEY = Symbol() as InjectionKey<
  ComputedRef<number>
>;

export function useZoomLevelProvider(zoom: Ref<number>) {
  provide(
    DIAGRAM_ZOOM_INJECTION_KEY,
    computed(() => zoom.value),
  );
}

export function useZoomLevel() {
  return inject(DIAGRAM_ZOOM_INJECTION_KEY);
}
