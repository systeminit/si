<template>
  <div class="flex items-center w-full h-6 status-bar">
    <div class="ml-1 text-xs text-gray-400">
      {{ application }}
    </div>
    <ChevronsRightIcon size="1.0x" class="text-gray-400" />
    <div class="text-xs text-gray-400">
      {{ system }}
    </div>
    <ChevronsRightIcon size="1.0x" class="text-gray-400" />
    <div class="text-xs text-gray-400">{{ node }} [{{ nodeType }}]</div>
    <div class="flex justify-end flex-grow mr-1">
      <div class="w-6 mr-1 text-xs" v-bind:class="modeClasses">
        {{ mode }}
      </div>

      <GitBranchIcon size="1.0x" class="mr-1 text-gray-400" />
      <div class="mr-1 text-xs text-gray-400">
        {{ changeSet }}
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { ChevronsRightIcon, GitBranchIcon } from "vue-feather-icons";
import { mapState } from "vuex";
import { camelCase } from "change-case";

export default Vue.extend({
  name: "StatusBar",
  components: {
    ChevronsRightIcon,
    GitBranchIcon,
  },
  computed: {
    modeClasses(): Record<string, boolean> {
      const result: Record<string, boolean> = {};
      if (this.mode == "edit") {
        result["mode-edit"] = true;
      } else {
        result["mode-view"] = true;
      }
      return result;
    },
    ...mapState({
      mode: (state: any): string => state.editor.mode,
      application: (state: any): string =>
        state.application.current?.name || "none",
      system: (state: any): string => state.system.current?.name || "none",
      node: (state: any): string => state.node.current?.name || "none",
      changeSet: (state: any): string =>
        state.changeSet.current?.name || "none",
      changeSetId: (state: any): string =>
        state.changeSet.current?.id || "none",
      nodeType(state: any): string {
        let typeName;
        if (this.node.name != "none") {
          if (this.changeSet == "none") {
            typeName = state.node.current?.display?.saved?.siStorable?.typeName;
          } else {
            if (state.node.current?.display[this.changeSetId]) {
              typeName =
                state.node.current?.display[this.changeSetId].siStorable
                  ?.typeName;
            } else {
              typeName =
                state.node.current?.display?.saved?.siStorable?.typeName;
            }
          }
        } else {
          typeName = "none";
        }
        if (!typeName) {
          typeName = "none";
        }
        return camelCase(typeName);
      },
    }),
  },
});
</script>

<style>
.status-bar {
  background-color: #1f2324;
}

.mode-edit {
  color: #d9a35e;
}
.mode-view {
  color: #3091c1;
}
</style>
