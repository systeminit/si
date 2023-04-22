/** TEMPORARY store test page - route accessible at /store-test */
<template>
  <div
    class="bg-neutral-900 w-full min-h-screen overflow-hidden flex flex-col text-white p-lg"
  >
    <Stack>
      <h2>Store test!</h2>
      <Inline>
        <VButton icon="plus-circle" tone="success" @click="addWidget"
          >Add widget</VButton
        >
        <!-- <VButton
          icon="minus-circle"
          tone="destructive"
          :disabled="numWidgets === 0"
          @click=""
        >
          Remove widget
        </VButton> -->
      </Inline>
      <CounterWidget
        v-for="i in _.keys(widgets)"
        :id="i"
        :key="i"
        @destroy="destroyWidget(i)"
      />
      <div>Count = {{ store.counter }}</div>
    </Stack>
  </div>
</template>

<script lang="ts" setup>
/* eslint-disable */
import { computed, reactive, ref } from "vue";
import * as _ from "lodash-es";
import { VButton, Stack, Inline } from "@si/vue-lib/design-system";
import { useCounterStore, useCounterStore2 } from "@/store/counter.store";
import CounterWidget from "./CounterWidget.vue";

const widgets = reactive({} as Record<number, true>);
const numWidgets = computed(() => _.keys(widgets).length);

const store = useCounterStore();
// const store = useCounterStore2();

let widgetIdCounter = 0;
function addWidget() {
  widgets[widgetIdCounter++] = true;
}
function destroyWidget(id: any) {
  console.log(`destroying widget - ${id}`);
  delete widgets[id];
}

// const count = computed(() => {
//   const store = useCounterStore();
//   return store.counter;
// });
</script>
