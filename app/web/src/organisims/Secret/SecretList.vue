<template>
  <div class="flex w-full h-full">
    <SiError
      v-if="placeholderShowError"
      :test="placeholderString"
      :message="placeholderString"
      @clear="placeholderFunc"
    />
    <div class="flex w-full h-full">
      <div class="flex flex-col w-full shadow-sm table-fixed">
        <div class="flex w-full text-sm font-medium text-gray-200 header">
          <div class="w-6/12 px-2 py-1 text-center align-middle table-border">
            Name
          </div>
          <div class="w-3/12 px-2 py-1 text-center table-border">Kind</div>
          <div class="w-3/12 px-2 py-1 text-center table-border">Type</div>
        </div>

        <div class="flex flex-col overflow-y-scroll text-xs text-gray-300">
          <div
            v-for="secret in placeholderSecretList"
            :key="secret.id"
            class="flex items-center row-item"
          >
            <div class="w-6/12 px-2 py-1 text-center">
              {{ secret.name }}
            </div>
            <div class="w-3/12 px-2 py-1 text-center">
              {{ placeholderLabelForKind(secret.kind) }}
            </div>
            <div class="w-3/12 px-2 py-1 text-center">
              {{ placeholderLabelForObjectType(secret.objectType) }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SiError from "@/atoms/SiError.vue";

const placeholderString = "ilikemybutt";
const placeholderFunc = () => {
  console.log(placeholderString);
};

const placeholderShowError = false;
const placeholderSecretList = [
  {
    id: placeholderString,
    name: placeholderString,
    kind: "DockerHub",
    objectType: "Credential",
  },
];
const placeholderLabelForKind = (kind: string) => {
  return kind;
};
const placeholderLabelForObjectType = (objectType: string) => {
  return objectType;
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
