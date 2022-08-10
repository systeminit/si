<template>
  <div v-if="funcId > 0" class="text-center">
    <SiTextBox2
      id="handler"
      v-model="editingFunc.modifiedFunc.handler"
      class="w-full"
      title="Entrypoint"
      @change="updateFunc"
    />
    <SiTextBox2
      id="name"
      v-model="editingFunc.modifiedFunc.name"
      title="Name"
      class="w-full"
      @change="updateFunc"
    />
  </div>
  <div v-else class="p-2 text-center text-neutral-400">
    Select a function to view its properties.
  </div>
</template>

<script setup lang="ts">
import SiTextBox2 from "@/atoms/SiTextBox2.vue";
import { toRef } from "vue";
import {
  EditingFunc,
  funcEdit$,
  funcStream$,
  nullEditingFunc,
} from "@/observable/func_editor";
import { map, combineLatestWith } from "rxjs/operators";
import { refFrom, fromRef } from "vuse-rx/src";

const props = defineProps<{
  funcId: number;
}>();

const funcId = toRef(props, "funcId", -1);
const funcId$ = fromRef<number>(funcId);

const editingFunc = refFrom<EditingFunc>(
  funcStream$.pipe(
    combineLatestWith(funcId$),
    map(
      ([editingFuncs, funcId]) =>
        editingFuncs?.find((f) => f.id == funcId) ?? nullEditingFunc,
    ),
  ),
  nullEditingFunc,
);

const updateFunc = () => {
  funcEdit$.next({
    type: "change",
    func: {
      ...editingFunc.value.modifiedFunc
    }
  });
}

</script>
