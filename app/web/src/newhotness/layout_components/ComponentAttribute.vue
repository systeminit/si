<template>
  <li class="flex flex-col">
    <template v-if="hasChildren">
      <AttributeChildLayout>
        <template #header>
          {{ displayName }}
        </template>
        <ul>
          <ComponentAttribute
            v-for="child in attributeTree.children.filter(filterMissingAtom)"
            :key="child.id"
            :attributeTree="child"
            @save="(path, id, value) => emit('save', path, id, value)"
          />
        </ul>
      </AttributeChildLayout>
    </template>
    <!-- arrays that have no children -->
    <template v-else-if="isArray">
      <AttributeChildLayout>
        <template #header>
          {{ displayName }}
        </template>
        <div class="p-xs">
          <VButton class="font-normal" tone="shade" variant="ghost" size="sm"
            >+ add {{ displayName }}</VButton
          >
        </div>
      </AttributeChildLayout>
    </template>
    <template v-else>
      <AttributeInput
        :displayName="displayName"
        :attributeTree="props.attributeTree"
        @save="(path, id, value) => emit('save', path, id, value)"
      />
    </template>
  </li>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { VButton } from "@si/vue-lib/design-system";
import { BifrostAttributeTree } from "@/workers/types/dbinterface";
import { filterMissingAtom } from "../util";
import AttributeChildLayout from "./AttributeChildLayout.vue";
import AttributeInput from "./AttributeInput.vue";

const props = defineProps<{
  attributeTree: BifrostAttributeTree;
}>();

// this handles objects and arrays but not empty arrays
const hasChildren = computed(() => props.attributeTree.children.length > 0);

const isArray = computed(() => props.attributeTree.prop?.kind === "array");

const displayName = computed(() => {
  if (props.attributeTree.attributeValue.key)
    return props.attributeTree.attributeValue.key;
  else return props.attributeTree.prop?.name || "XXX";
});

const emit = defineEmits<{
  (e: "save", path: string, id: string, value: string): void;
}>();
</script>
