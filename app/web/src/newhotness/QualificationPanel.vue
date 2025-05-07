<template>
  <ul class="p-xs flex flex-col gap-xs">
    <QualificationView
      v-for="item in items"
      :key="item.av_id"
      :qualification="item"
    />
  </ul>
</template>

<script lang="ts" setup>
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import {
  BifrostAttributeTree,
  BifrostComponent,
} from "@/workers/types/dbinterface";
import QualificationView from "@/newhotness/QualificationView.vue";
import { AttributeValueId } from "@/store/status.store";
import { QualificationStatus } from "@/store/qualifications.store";

export interface QualItem {
  name: string;
  message: string;
  result: QualificationStatus;
  av_id?: AttributeValueId;
}

const props = defineProps<{
  attributeValueId: string;
  component: BifrostComponent;
}>();

const attributeValueId = computed(() => props.attributeValueId);

const key = useMakeKey();
const args = useMakeArgs();
const attributeTreeQuery = useQuery<BifrostAttributeTree | null>({
  queryKey: key("AttributeTree", attributeValueId),
  queryFn: async () =>
    await bifrost<BifrostAttributeTree>(
      args("AttributeTree", attributeValueId.value),
    ),
});

const root = computed(() => attributeTreeQuery.data.value);

const domain = computed(() =>
  root.value?.children.find((c) => c.prop?.name === "domain"),
);

const quals = computed(() =>
  root.value?.children.find((c) => c.prop?.name === "qualification"),
);

const validations = computed(() => {
  const output: BifrostAttributeTree[] = [];
  if (!domain.value) return output;
  const getValidations = (c: BifrostAttributeTree) =>
    c.validation?.status && c.validation?.status !== "Success";
  const walking = [...domain.value.children];
  // walk all the children and find if they match
  while (walking.length > 0) {
    const c = walking.shift();
    if (!c) break;
    walking.push(...c.children);

    const v = getValidations(c);
    if (v) {
      output.push(c);
    }
  }

  return output;
});

// TODO(Wendy) - this is very annoying!
// Why are statuses sometimes capitalized and sometimes not?!?!?!
const fixStatus = (status: string): QualificationStatus => {
  switch (status) {
    case "Success":
      return "success";
    case "Failure":
      return "failure";
    case "Error":
      return "failure";
    default:
      return "running";
  }
};

const items = computed<QualItem[]>(() => {
  const qualItems =
    quals.value?.children.map((c): QualItem => {
      const name = c.attributeValue.key ?? "";
      const message =
        c.children.find((_c) => _c.prop?.name === "message")?.attributeValue
          .value ?? "";
      const result = (c.children.find((_c) => _c.prop?.name === "result")
        ?.attributeValue.value ?? "running") as QualificationStatus;
      const av_id = c.id;
      return {
        name,
        message,
        result,
        av_id,
      };
    }) ?? [];

  validations.value.forEach((v) => {
    if (v.validation && v.prop) {
      qualItems.push({
        name: v.prop.name,
        message: v.validation.message || "",
        result: fixStatus(v.validation.status),
      });
    }
  });

  return qualItems;
});
</script>
