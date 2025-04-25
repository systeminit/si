<template>
  <section class="grid gap-md h-[100%]">
    <div>
      <!-- filter / top header -->
      <InstructiveVormInput
        classes="py-2"
        :activeClasses="
          clsx(themeClasses('border-action-500', 'border-action-300'))
        "
        :pills="['Up', 'Down']"
        instructions="to navigate"
      >
        <template #default="slotProps">
          <VormInput
            ref="inputRef"
            v-model="searchString"
            autocomplete="off"
            :class="slotProps.class"
            noStyles
            @focus="slotProps.focus"
            @blur="slotProps.blur"
          />
        </template>
      </InstructiveVormInput>
    </div>
    <div>
      <!-- right header -->
      right header
    </div>
    <div class="scrollable tilegrid">
      <!-- body -->
      <ComponentGridTile
        v-for="component in componentViewList"
        :key="component.id"
        :component="component"
        @dblclick="componentNavigate"
      />
    </div>
    <div class="grid grid-rows-subgrid gap-sm" :style="collapsingStyles">
      <CollapsingGridItem ref="actions">
        <!-- TODO(nick,wendy): remove or replace the crude counter. This is solely used for development of the new UI. -->
        <template #header>Actions ({{ actionViewList.length }})</template>
        <ul class="actions list">
          <!-- eslint-disable-next-line vue/no-unused-vars -->
          <li v-for="action in actionViewList" :key="action.id" class="item">
            <StatusIndicatorIcon
              :status="action.kind.toString()"
              type="action"
            />
            <h2>{{ action.name }}</h2>
            <h3>
              {{ action.componentName ?? "- component name not found -" }}
            </h3>
            <DetailsPanelMenuIcon
              @click="
                (e) => {
                  contextMenuRef?.open(e, false);
                }
              "
            />
          </li>
        </ul>
      </CollapsingGridItem>
      <CollapsingGridItem ref="history">
        <template #header>History</template>
        <ul>
          <!-- eslint-disable-next-line vue/no-unused-vars -->
          <li v-for="(_, idx) in new Array(15)" :key="idx">Item: {{ idx }}</li>
        </ul>
      </CollapsingGridItem>
    </div>
    <div class="place-items-end">
      <!-- footer -->
      <div>
        <VButton label="Add a component" pill="Cmd + A" tone="action" />
      </div>
    </div>

    <DropdownMenu ref="contextMenuRef" :forceAbove="false" forceAlignRight>
      <h5 class="text-neutral-400 pl-2xs">ACTIONS:</h5>
      <DropdownMenuItem>Foo</DropdownMenuItem>
      <DropdownMenuItem>Bar</DropdownMenuItem>
      <DropdownMenuItem>Baz</DropdownMenuItem>
    </DropdownMenu>
  </section>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import { useRouter, useRoute } from "vue-router";
import {
  themeClasses,
  VormInput,
  VButton,
  DropdownMenu,
  DropdownMenuItem,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useQuery } from "@tanstack/vue-query";
import { bifrost, makeArgs, makeKey } from "@/store/realtime/heimdall";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";
import { ActionViewList, ComponentViewList } from "@/workers/types/dbinterface";
import { collapsingGridStyles } from "./util";
import CollapsingGridItem from "./layout_components/CollapsingGridItem.vue";
import InstructiveVormInput from "./layout_components/InstructiveVormInput.vue";
import DetailsPanelMenuIcon from "./layout_components/DetailsPanelMenuIcon.vue";
import ComponentGridTile from "./ComponentGridTile.vue";

const actions = ref<typeof CollapsingGridItem>();
const history = ref<typeof CollapsingGridItem>();

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

// TODO this is where you do a tanStack query like this:
// https://github.com/systeminit/si/blob/main/app/web/src/workers/webworker.ts#L818
// const components = [];

const actionViewListRaw = useQuery<ActionViewList | null>({
  queryKey: makeKey("ActionViewList"),
  queryFn: async () =>
    await bifrost<ActionViewList>(makeArgs("ActionViewList")),
});
const actionViewList = computed(
  () => actionViewListRaw.data.value?.actions ?? [],
);

const componentViewListRaw = useQuery<ComponentViewList | null>({
  queryKey: makeKey("ComponentViewList"),
  queryFn: async () =>
    await bifrost<ComponentViewList>(makeArgs("ComponentViewList")),
});
const componentViewList = computed(
  () => componentViewListRaw.data.value?.components ?? [],
);

const searchString = ref("searching...");

const collapsingStyles = computed(() =>
  collapsingGridStyles([actions.value?.openState, history.value?.openState]),
);

const router = useRouter();
const route = useRoute();
const componentNavigate = () => {
  const params = { ...route.params };
  params.componentId = "123";
  router.push({
    name: "new-hotness-component",
    params,
  });
};
</script>

<style lang="css" scoped>
section.grid {
  grid-template-columns: minmax(0, 70%) minmax(0, 30%);
  grid-template-rows: 4rem 1fr 3rem;
}
</style>
