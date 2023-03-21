/* eslint-disable no-console */

import { defineStore } from "pinia";
import { watch } from "vue";
import { addStoreHooks } from "@si/vue-lib";

export const useCounterStore = addStoreHooks(
  defineStore("counter", {
    state: () => ({
      counter: 20,
      foo: 111,
    }),
    getters: {
      counterX2: (state) => state.counter * 2,
    },
    actions: {
      increment() {
        this.counter++;
      },
      decrement() {
        this.counter--;
      },
      reset() {
        this.counter = 0;
      },
    },
    onActivated() {
      console.log("counter store activated");
      const cleanupAlertWatch = watch(
        () => this.counter,
        () => {
          // eslint-disable-next-line no-alert
          if (this.counter === 5) alert("counter = 5!");
        },
      );
      return function onDeactivated() {
        console.log("counter store deactivated");
        cleanupAlertWatch();
      };
    },
  }),
);

export const useCounterStore2 = addStoreHooks(
  defineStore("indirect-counter", {
    state: () => ({
      foo: 1,
    }),
    getters: {
      counter: () => {
        const counterStore = useCounterStore();
        return counterStore.counter;
      },
    },
    onActivated() {
      console.log("indirect counter store activated");
      return function onDeactivated() {
        console.log("indirect counter store deactivated");
      };
    },
  }),
);
