<template>
  <div class="overflow-auto w-full h-full">
    <SiTabGroup>
      <template #tabs>
        <SiTabHeader v-for="func in loadedFuncs" :key="func.id">{{
          func.name
        }}</SiTabHeader>
      </template>
      <template #panels>
        <TabPanel v-for="func in loadedFuncs" :key="func.id">
          <pre>{{ func.code }}</pre>
        </TabPanel>
      </template>
    </SiTabGroup>
    <!--   <div
      ref="editorMount"
      class="w-full h-full"
      @keyup.stop
      @keydown.stop
    ></div> -->
  </div>
</template>

<script lang="ts" setup>
import { ref, onMounted, toRef } from "vue";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { defaultKeymap } from "@codemirror/commands";
import { refFrom, fromRef } from "vuse-rx/src";
import { combineLatest, iif, of } from "rxjs";
import { switchMap, tap } from "rxjs/operators";
import { FuncService } from "@/service/func";
import { GetFuncResponse, nullFunc } from "@/service/func/get_func";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import { TabPanel } from "@headlessui/vue";

const props = defineProps<{
  selectedFuncId: number;
}>();

const selectedFuncId = toRef(props, "selectedFuncId", 0);
const selectedFuncId$ = fromRef(selectedFuncId, { immediate: true });

const loadedFuncs = ref<{ [key: number]: GetFuncResponse }>({});

const editorMount = ref();
const view = ref<EditorView | undefined>();

const mountEditor = () => {
  let startState = EditorState.create({
    doc: "",
    extensions: [keymap.of(defaultKeymap)],
  });

  view.value = new EditorView({
    state: startState,
    parent: editorMount.value,
  });
};

onMounted(() => {
  if (editorMount.value) {
    console.log("mounting editor");
    mountEditor();
  }
});

refFrom<GetFuncResponse>(
  combineLatest([selectedFuncId$]).pipe(
    switchMap(([selectedFuncId]) =>
      iif(
        () => selectedFuncId > 0,
        FuncService.getFunc({ id: selectedFuncId }),
        of(nullFunc),
      ),
    ),
    tap((func) => {
      loadedFuncs.value[func.id] = func;
    }),
  ),
  nullFunc,
);
</script>
