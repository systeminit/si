import { vi } from "vitest";
import { createTestingPinia } from "@pinia/testing";
import { piniaHooksPlugin } from "@si/vue-lib/pinia";
import { VueQueryPlugin } from "@tanstack/vue-query";
import FloatingVue from "floating-vue";
import router from "@/router";

export const plugins = [
  FloatingVue,
  VueQueryPlugin,
  router,
  createTestingPinia({
    plugins: [piniaHooksPlugin],
    createSpy: vi.fn,
    initialState: {
      featureFlagStore: {
        DEFAULT_SUBS: true,
      },
    },
  }),
];

// handles any window.location code
Object.defineProperty(window, "location", {
  writable: true,
  value: {
    ...window.location, // Keep existing properties if needed
    assign: vi.fn(),
    replace: vi.fn(),
    href: "http://localhost/", // Set a default href for testing
  },
});
