<template>
  <li
    :class="
      clsx(
        'flex flex-row items-center gap-xs p-2xs border-x border-b',
        themeClasses('border-neutral-200', 'border-neutral-600'),
      )
    "
  >
    <IconButton icon="play" :requestStatus="request" @click="runPrototype()" />
    <span>{{ `Run ${props.prototype.label}` }}</span>
    <div
      :class="
        clsx(
          'ml-auto mr-2xs hover:underline font-bold select-none cursor-pointer',
          themeClasses('text-action-500', 'text-action-300'),
        )
      "
      @click.stop="onClickView"
    >
      view
    </div>
  </li>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { IconButton, themeClasses } from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import { useFuncStore, MgmtPrototype } from "@/store/func/funcs.store";
import { useComponentsStore } from "@/store/components.store";
import { ComponentId } from "@/api/sdf/dal/component";

const funcStore = useFuncStore();
const componentsStore = useComponentsStore();
const router = useRouter();

const props = defineProps<{
  prototype: MgmtPrototype;
  componentId: ComponentId;
}>();

const request = funcStore.getRequestStatus(
  "RUN_PROTOTYPE",
  props.prototype.managementPrototypeId,
  props.componentId,
);

const runPrototype = () => {
  funcStore.RUN_PROTOTYPE(
    props.prototype.managementPrototypeId,
    props.componentId,
  );
};

function onClickView() {
  router.push({
    name: "workspace-lab-assets",
    query: {
      s: `a_${componentsStore.selectedComponent?.schemaVariantId}|f_${props.prototype.funcId}`,
    },
  });
}
</script>
