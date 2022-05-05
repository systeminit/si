<template>
  <div class="flex items-center w-full h-6 pt-1 pb-1 status-bar">
    <div class="flex items-center">
      <div v-if="application" class="ml-1 text-xs application-name">
        {{ application.name }}
      </div>
    </div>

    <div
      v-if="editMode || changeSet"
      class="flex items-center justify-end flex-grow mr-1"
    >
      <VueFeather
        v-if="changeSet"
        type="git-branch"
        size="0.75rem"
        class="mr-1 changeset-icon"
      />
      <VueFeather
        v-else
        type="git-commit"
        size="0.75rem"
        class="mr-1 changeset-icon"
      />
      <div v-if="changeSet" class="mr-1 text-xs changeset-name">
        {{ changeSet.name }}
      </div>
      <div v-else class="mr-1 text-xs changeset-name">latest</div>
      <div class="mr-1 text-xs changeset-icon">|</div>
      <div v-if="editMode" class="w-6 mr-1 text-xs" :class="editModeClasses()">
        edit
      </div>
      <div v-else class="w-6 mr-1 text-xs" :class="editModeClasses()">read</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { refFrom } from "vuse-rx";

import VueFeather from "vue-feather";
import { ApplicationService } from "@/service/application";
import { ChangeSetService } from "@/service/change_set";
import { Component } from "@/api/sdf/dal/component";
import { ChangeSet } from "@/api/sdf/dal/change_set";

const application = refFrom<Component | null>(
  ApplicationService.currentApplication(),
);
const changeSet = refFrom<ChangeSet | null>(
  ChangeSetService.currentChangeSet(),
);
const editMode = refFrom<boolean>(ChangeSetService.currentEditMode());

const editModeClasses = () => {
  let classes: Record<string, boolean> = {};

  if (editMode.value) {
    classes["mode-edit"] = true;
  } else {
    classes["mode-view"] = true;
  }
  return classes;
};
</script>

<style scoped>
.status-bar {
  background-color: #1f2324;
  border-bottom: 1px solid #2a2a2a;
}

.application-name {
  color: #ffd2d3;
}

.system-name {
  color: #e0ffd2;
}

.node-name {
  color: #d2f6ff;
}

.node-type {
  color: #96eaff;
}

.changeset-icon {
  color: #7d9c9a;
}

.changeset-name {
  color: #fff5c8;
}

.mode-edit {
  /* color: #d9a35e; */
  color: #ffbf6f;
}

.mode-view {
  /* color: #3091c1; */
  color: #40bfff;
}
</style>
