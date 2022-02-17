<template>
  <div class="flex flex-col w-full">
    <!--    <SiError-->
    <!--      v-if="showError"-->
    <!--      :test="placeholderString"-->
    <!--      :message="placeholderString"-->
    <!--      :success="true"-->
    <!--      @clear="placeholderFunc"-->
    <!--    />-->
    <div class="flex flex-row items-center w-full pb-2">
      <div class="w-1/2 pr-2 text-sm text-right text-gray-400 align-middle">
        <label for="secret-name-textbox">Secret Name:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          id="secret-name-textbox"
          v-model="nameField"
          size="xs"
          name="secretName"
          placeholder="secret name"
          :is-show-type="false"
          required
        />
      </div>
    </div>
    <div class="flex flex-row items-center w-full pb-2">
      <div class="w-1/2 pr-2 text-sm text-right text-gray-400 align-middle">
        <label for="secret-password-textbox">Secret Kind:</label>
      </div>
      <div class="w-1/2 align-middle">
        <SiSelect
          id="secret-password-textbox"
          v-model="selectedSecretKindName"
          size="xs"
          name="secretKind"
          :options="secretKindOptions"
          required
        />
      </div>
    </div>

    <SecretCreateFields
      v-if="selectedSecretKind"
      :secret-kind="selectedSecretKind"
      @input="placeholderFunc"
    />

    <div class="flex justify-end w-full">
      <div class="pr-2">
        <SiButton
          size="xs"
          label="Cancel"
          kind="cancel"
          :icon="null"
          @click="returnToListView"
        />
      </div>
      <div>
        <!-- NOTE(nick): disable the create button if the user exits edit mode. We will still keep
        this component alive in case that was an accident.
        -->
        <SiButton
          size="xs"
          label="Create"
          kind="save"
          :icon="null"
          :disabled="!editMode"
          @click="returnToListView"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SiButton from "@/atoms/SiButton.vue";
// import SiError from "@/atoms/SiError.vue";
import SiSelect, { SelectPropsOption } from "@/atoms/SiSelect.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import { computed, ref } from "vue";
import { SecretService } from "@/service/secret";
import { SecretKind } from "@/api/sdf/dal/secret";
import SecretCreateFields from "@/organisims/Secret/SecretCreateFields.vue";
import { refFrom } from "vuse-rx";
import { ChangeSetService } from "@/service/change_set";

defineProps<{
  modelValue: string;
}>();

const editMode = refFrom<boolean>(ChangeSetService.currentEditMode());

// NOTE(nick): this is a tad "hacky", but ensures we get the actual kind for a given name.
// Reactivity was not behaving as intended with "SecretKind", so we are using "string" instead.
// Even if we switch back to using "SecretKind", we need to ensure our core variable defaults to
// "unset" or equivalent.
const selectedSecretKindName = ref<string>("");
const selectedSecretKind = computed((): SecretKind | null => {
  if (!selectedSecretKindName.value) {
    return null;
  }
  for (const kind of secretKinds.value) {
    if (kind.name === selectedSecretKindName.value) {
      return kind;
    }
  }
  return null;
});

const secretKinds = computed((): SecretKind[] => {
  return SecretService.listSecretKinds();
});

// These are used for options to display in the creation dropdown.
// Our first entry in the array is the "unset" dropdown option.
const secretKindOptions = computed((): SelectPropsOption[] => {
  let options: SelectPropsOption[] = [{ label: "", value: "" }];
  for (const kind of secretKinds.value) {
    options.push({ label: kind.name, value: kind.name });
  }
  return options;
});

const nameField = "";
const placeholderFunc = () => {
  console.log(nameField);
};

// Use "emits" to switch back to "list" view.
const emits = defineEmits(["update:modelValue"]);
const returnToListView = () => {
  emits("update:modelValue", "list");
};
</script>

<style scoped>
.background {
  background-color: #1e1e1e;
}

.header {
  background-color: #3a3d40;
}

.row-item {
  background-color: #262626;
}

.row-item:nth-child(odd) {
  background-color: #2c2c2c;
}

.table-border {
  border-bottom: 1px solid #46494d;
}
</style>
