<template>
  <div class="flex flex-row gap-xs items-end p-xs justify-end w-full">
    <VormInput
      label="CLI"
      type="text"
      disabled
      defaultValue="aws"
      :maxLength="3"
      @enterPressed="generateAwsFunction"
    />
    <VormInput
      v-model="awsCommand.command"
      label="Command"
      type="text"
      :disabled="generating"
      @enterPressed="generateAwsFunction"
    />
    <VormInput
      v-model="awsCommand.subcommand"
      label="Subcommand"
      type="text"
      :disabled="generating"
      @enterPressed="generateAwsFunction"
    />
    <VButton
      v-tooltip="'Generate Schema From AWS Command'"
      class="mb-[2px]"
      :loading="generating"
      loadingIcon="sparkles"
      loadingText="Generating ..."
      :requestStatus="generateRequestStatus"
      label="Generate"
      size="sm"
      @click="generateAwsFunction"
    />
  </div>
</template>

<script lang="ts" setup>
import { reactive, PropType, computed } from "vue";
import { VormInput, VButton } from "@si/vue-lib/design-system";
import { useFuncStore, AwsCliCommand } from "@/store/func/funcs.store";
import { SchemaVariantId } from "@/api/sdf/dal/schema";
import { FuncId } from "@/api/sdf/dal/func";

const funcStore = useFuncStore();

const props = defineProps({
  funcId: { type: String as PropType<FuncId>, required: true },
  schemaVariantId: {
    type: String as PropType<SchemaVariantId>,
    required: true,
  },
  generatingCommand: { type: Object as PropType<AwsCliCommand> },
});

/** The AWS command entered in the form */
const awsCommand = reactive({
  command: props.generatingCommand?.command ?? "sqs",
  subcommand: props.generatingCommand?.subcommand ?? "create-queue",
});

/** Status of the request */
const generateRequestStatus = funcStore.getRequestStatus(
  "GENERATE_AWS_FUNCTION",
  props.funcId,
);
const generating = computed(
  () => generateRequestStatus.value.isPending || !!props.generatingCommand,
);

/** Actually trigger schema generation */
const generateAwsFunction = async () => {
  if (generating.value) {
    return;
  }
  await funcStore.GENERATE_AWS_FUNCTION(
    props.funcId,
    awsCommand,
    props.schemaVariantId,
  );
};
</script>
