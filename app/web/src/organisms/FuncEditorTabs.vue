<template>
  <div class="overflow-auto w-full h-full">
    <SiTabGroup :selected-index="selectedTab" @change="changeTab">
      <template #tabs>
        <SiTabHeader v-for="func in funcList" :key="func.id">{{
          func.name
        }}</SiTabHeader>
      </template>
      <template #panels>
        <TabPanel v-for="func in funcList" :key="func.id">
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
import { onMounted, ref, toRef } from "vue";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { defaultKeymap } from "@codemirror/commands";
import { refFrom, fromRef } from "vuse-rx/src";
import { combineLatest, iif, of } from "rxjs";
import { switchMap, tap } from "rxjs/operators";
import { FuncService } from "@/service/func";
import { GetFuncResponse } from "@/service/func/get_func";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import { TabPanel } from "@headlessui/vue";

const props = defineProps<{
  selectedFuncId: number;
}>();

const emits = defineEmits<{
  (e: "selectedFunc", v: number): void;
}>();

const selectFunc = (funcId: number) => {
  emits("selectedFunc", funcId);
};

const selectedFuncId = toRef(props, "selectedFuncId", 0);
const selectedFuncId$ = fromRef(selectedFuncId, { immediate: true });

const selectedTab = ref(0);

const funcList = ref<GetFuncResponse[]>([]);
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

const changeTab = (index: number) => {
  selectFunc(funcList.value[index].id);
};

const findTabIndexForFunc = (func: GetFuncResponse) =>
  funcList.value.findIndex((fn) => fn.id == func.id);

const onFuncSelected = (func: GetFuncResponse) => {
  loadedFuncs.value[func.id] = func;
  let tabIndex = findTabIndexForFunc(func);
  if (tabIndex === -1) {
    funcList.value.push(func);
    selectedTab.value = funcList.value.length - 1;
  } else {
    selectedTab.value = tabIndex;
  }
};

refFrom<GetFuncResponse | undefined>(
  combineLatest([selectedFuncId$]).pipe(
    switchMap(([selectedFuncId]) =>
      iif(
        () => selectedFuncId > 0,
        FuncService.getFunc({ id: selectedFuncId }),
        of(undefined),
      ),
    ),
    tap((func) => {
      func && onFuncSelected(func);
    }),
  ),
  undefined,
);
</script>
