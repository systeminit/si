<template>
  <div
    class="flex items-center w-full h-6 pt-1 pb-1 status-bar"
    data-testid="status-bar"
  >
    <div class="flex items-center">
      <div class="ml-1 text-xs application-name" v-if="applicationName">
        {{ applicationName }}
      </div>

      <!-- <ChevronsRightIcon size="0.75x" class="text-gray-400" v-if="systemName" />
      <div class="text-xs system-name" v-if="systemName">
        cs:{{ systemName }} "context: application system(as) | computing system(cs)"
      </div>

      <ChevronsRightIcon
        size="0.75x"
        class="text-gray-400"
        v-if="nodeName || nodeType"
      />
      <div v-if="nodeName || nodeType">
        <span class="text-xs node-name" v-if="nodeName">{{ nodeName }}</span>
        <span class="ml-1 text-xs node-type" v-if="nodeType"
          >[{{ nodeType }}]</span
        >
      </div> -->
    </div>

    <div
      class="flex items-center justify-end flex-grow mr-1"
      v-if="editMode || changeSetName"
    >
      <GitBranchIcon
        size="0.75x"
        class="mr-1 changeset-icon"
        v-if="changeSetName"
      />
      <GitCommitIcon size="0.75x" class="mr-1 changeset-icon" v-else />
      <!-- <div class="mr-1 text-xs changeset-icon" v-else>
        |
      </div> -->
      <div class="mr-1 text-xs changeset-name" v-if="changeSetName">
        {{ changeSetName }}
      </div>
      <div class="mr-1 text-xs changeset-name" v-else>
        latest
      </div>
      <div class="mr-1 text-xs changeset-icon" v-if="editMode">
        |
      </div>
      <div class="w-6 mr-1 text-xs" :class="editModeClasses()" v-if="editMode">
        {{ editMode }}
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import {
  // ChevronsRightIcon,
  GitBranchIcon,
  GitCommitIcon,
} from "vue-feather-icons";
import { Entity } from "@/api/sdf/model/entity";
import { changeSet$, editMode$ } from "@/observables";
import { switchMap } from "rxjs/operators";
import { from } from "rxjs";

export default Vue.extend({
  name: "StatusBar",
  components: {
    // ChevronsRightIcon,
    GitBranchIcon,
    GitCommitIcon,
  },
  props: {
    application: { type: Object as PropType<Entity> },
  },
  subscriptions(): Record<string, any> {
    return {
      changeSetName: changeSet$.pipe(
        switchMap(changeSet => from([changeSet?.name || null])),
      ),
      editMode: editMode$,
    };
  },
  computed: {
    applicationName(): string | undefined {
      if (this.application) {
        return this.application.name;
      } else {
        return undefined;
      }
    },
  },
  methods: {
    editModeClasses(): Record<string, any> {
      let classes: Record<string, any> = {};

      // @ts-ignore
      if (this.editMode) {
        classes["mode-edit"] = true;
      } else {
        classes["mode-view"] = true;
      }
      return classes;
    },
  },
});
</script>

<style>
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
