/** TEMPORARY store test page - route accessible at /store-test */
<template>
  <div
    class="bg-neutral-900 w-full min-h-screen overflow-hidden flex flex-col text-white p-lg"
  >
    <Stack>
      <h2>Store test!</h2>
      <Inline>
        <VButton2 icon="plus-circle" tone="success" @click="addWidget"
          >Add widget</VButton2
        >
        <!-- <VButton2
          icon="minus-circle"
          tone="destructive"
          :disabled="numWidgets === 0"
          @click=""
        >
          Remove widget
        </VButton2> -->
      </Inline>
      <CounterWidget
        v-for="i in _.keys(widgets)"
        :id="i"
        :key="i"
        @destroy="destroyWidget(i)"
      />
      <div>Count = {{ count }}</div>
    </Stack>
  </div>
</template>

<script lang="ts" setup>
/* eslint-disable */
import { computed, reactive, ref } from "vue";
import _ from "lodash";
import VButton2 from "@/ui-lib/VButton2.vue";
import { useCounterStore, useCounterStore2 } from "@/store/counter.store";
import Stack from "../../ui-lib/layout/Stack.vue";
import Inline from "../../ui-lib/layout/Inline.vue";
import CounterWidget from "./CounterWidget.vue";

const widgets = reactive({} as Record<number, true>);
const numWidgets = computed(() => _.keys(widgets).length);

// const store = useCounterStore();
// const store = useCounterStore2();
// const count = computed(() => store.counter);
const count = computed(() => 999);

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
