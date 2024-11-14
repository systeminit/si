<template>
  <div class="flex flex-row gap-xs items-end p-xs justify-end w-full">
    <VormInput
      v-model="aws"
      label="CLI"
      type="text"
      disabled
      :maxLength="3"
      @enterPressed="generateAwsFunction"
    />
    <VormInput
      v-model="awsCommand.command"
      label="Command"
      type="text"
      :disabled="!!isLocked || generateRequestStatus.isPending"
      @enterPressed="generateAwsFunction"
    />
    <VormInput
      v-model="awsCommand.subcommand"
      label="Subcommand"
      type="text"
      :disabled="!!isLocked || generateRequestStatus.isPending"
      @enterPressed="generateAwsFunction"
    />
    <VButton
      class="mb-3xs"
      loadingIcon="sparkles"
      :loadingText="`Generating ${kind?.description}...`"
      :requestStatus="generateRequestStatus"
      :label="
        isLocked
          ? `Locked (unlock before generating)`
          : `Generate ${kind?.description}`
      "
      size="sm"
      :disabled="!!isLocked"
      @click="generateAwsFunction"
    />
  </div>
</template>

<script lang="ts" setup>
import { reactive, watch } from "vue";
import { VormInput, VButton } from "@si/vue-lib/design-system";
import {
  useFuncStore,
  GenerateAwsFunctionKind,
  AwsCliCommand,
} from "@/store/func/funcs.store";
import { SchemaVariantId } from "@/api/sdf/dal/schema";
import { FuncId } from "@/api/sdf/dal/func";

const funcStore = useFuncStore();

const props = defineProps<{
  funcId: FuncId;
  schemaVariantId: SchemaVariantId;
  kind: GenerateAwsFunctionKind;
  isLocked?: boolean;
}>();

const generateRequestStatus = funcStore.getRequestStatus(
  "GENERATE_AWS_FUNCTION",
  props.funcId,
);

const aws = "aws";

/** Command and subcommand we're editing */
const awsCommand = reactive({ command: "", subcommand: "" });
// Set the command if the example changes or we start generating a command
// (We do not reset the command when generation stops; we want to leave it as-is)
function setCommand(command?: Readonly<AwsCliCommand>) {
  if (command) Object.assign(awsCommand, command);
}
watch(() => props.kind?.exampleCommand, setCommand, { immediate: true });
watch(() => funcStore.generatingFuncCode[props.funcId], setCommand, {
  immediate: true,
});

/** Actually trigger schema generation */
function generateAwsFunction() {
  funcStore.GENERATE_AWS_FUNCTION(
    props.funcId,
    awsCommand,
    props.schemaVariantId,
  );
}
</script>
