<template>
  <div class="flex flex-row gap-xs items-end p-xs justify-end w-full">
    <VormInput
      v-model="awsCommand.cli"
      label="CLI"
      type="text"
      disabled
      :maxLength="3"
      @enterPressed="generateAwsAssetSchema"
    />
    <VormInput
      v-model="awsCommand.command"
      label="Command"
      type="text"
      :disabled="generateRequestStatus.isPending"
      @enterPressed="generateAwsAssetSchema"
    />
    <VormInput
      v-model="awsCommand.subcommand"
      label="Subcommand"
      type="text"
      :disabled="generateRequestStatus.isPending"
      @enterPressed="generateAwsAssetSchema"
    />
    <VButton
      v-tooltip="'Generate Schema From AWS Command'"
      class="mb-[2px]"
      :loading="generateRequestStatus.isPending"
      loadingIcon="sparkles"
      loadingText="Generating ..."
      :requestStatus="generateRequestStatus"
      label="Generate"
      size="sm"
      @click="generateAwsAssetSchema"
    />
  </div>
</template>

<script lang="ts" setup>
import { reactive, PropType } from "vue";
import { VormInput, VButton } from "@si/vue-lib/design-system";
import { useAssetStore } from "@/store/asset.store";
import { SchemaVariant } from "@/api/sdf/dal/schema";

const assetStore = useAssetStore();

const props = defineProps({
  /** Asset we're generating for */
  asset: { type: Object as PropType<SchemaVariant>, required: true },
});

/** The AWS command entered in the form */
const awsCommand = reactive({
  cli: "aws",
  command: "sqs",
  subcommand: "create-queue",
});

/** Status of the  */
const generateRequestStatus = assetStore.getRequestStatus(
  "GENERATE_AWS_ASSET_SCHEMA",
  props.asset.schemaVariantId,
);

/** Actually trigger schema generation */
const generateAwsAssetSchema = async () => {
  await assetStore.GENERATE_AWS_ASSET_SCHEMA(
    props.asset.schemaVariantId,
    awsCommand.command,
    awsCommand.subcommand,
  );
};
</script>
