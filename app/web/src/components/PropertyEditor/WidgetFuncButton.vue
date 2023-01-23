<template>
  <Menu>
    <div class="ml-auto">
      <MenuButton>
        <VButton
          button-rank="primary"
          button-type="success"
          icon="plus"
          icon-right="chevron--down"
          label="f(x)"
          size="sm"
        />
        <MenuItems
          class="z-30 absolute right-4 mt-2 rounded bg-white dark:bg-black shadow-lg border focus:outline-none overflow-hidden"
        >
          <MenuItem
            v-if="hasExistingCustomFunction && !props.func.isBuiltin"
            as="button"
            :class="menuItemClasses"
            @click="routeToFunc(props.func.id)"
          >
            Modify current component attribute function
          </MenuItem>
          <MenuItem
            v-if="!hasExistingCustomFunction || props.func.isBuiltin"
            as="button"
            :class="menuItemClasses"
            @click="onCreateAttributeFunc"
          >
            Attach new function for component attribute
          </MenuItem>
          <MenuItem
            v-if="hasExistingCustomFunction && !props.func.isBuiltin"
            as="button"
            :class="menuItemClasses"
          >
            Detach function
          </MenuItem>
        </MenuItems>
      </MenuButton>
    </div>
  </Menu>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import { Menu, MenuButton, MenuItem, MenuItems } from "@headlessui/vue";
import { FuncWithPrototypeContext } from "@/api/sdf/dal/property_editor";
import VButton from "@/components/VButton.vue";
import { isCustomizableFuncKind } from "@/api/sdf/dal/func";
import { useRouteToFunc } from "@/utils/useRouteToFunc";

const routeToFunc = useRouteToFunc();

const menuItemClasses =
  "w-full flex relative flex-row whitespace-nowrap items-center py-2 px-4 cursor-pointer gap-2 hover:bg-action-500 hover:text-white";

const hasExistingCustomFunction = computed(() =>
  isCustomizableFuncKind(props.func.variant),
);

const props = defineProps<{
  func: FuncWithPrototypeContext;
  valueId: string;
}>();

const emits = defineEmits<{
  (
    e: "createAttributeFunc",
    currentFunc: FuncWithPrototypeContext,
    valueId: string,
  ): void;
}>();

const onCreateAttributeFunc = () =>
  emits("createAttributeFunc", props.func, props.valueId);
</script>
