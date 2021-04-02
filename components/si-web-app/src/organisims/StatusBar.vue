<template>
  <div
    class="flex items-center w-full h-6 pt-1 pb-1 status-bar"
    data-testid="status-bar"
  >
    <div class="ml-1 text-xs text-gray-400" v-if="applicationName">
      applications/{{ applicationName }}
    </div>

    <ChevronsRightIcon size="1.0x" class="text-gray-400" v-if="systemName" />
    <div class="text-xs text-gray-400" v-if="systemName">
      {{ systemName }}
    </div>

    <ChevronsRightIcon
      size="1.0x"
      class="text-gray-400"
      v-if="nodeName || nodeType"
    />
    <div class="text-xs text-gray-400" v-if="nodeName || nodeType">
      <span v-if="nodeName">{{ nodeName }}</span>
      <span v-if="nodeType">[{{ nodeType }}]</span>
    </div>

    <div
      class="flex justify-end flex-grow mr-1"
      v-if="editMode || changeSetName"
    >
      <div class="w-6 mr-1 text-xs" v-if="editMode">
        {{ editMode }}
      </div>

      <GitBranchIcon
        size="1.0x"
        class="mr-1 text-gray-400"
        v-if="changeSetName"
      />
      <div class="mr-1 text-xs text-gray-400" v-if="changeSetName">
        {{ changeSetName }}
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { ChevronsRightIcon, GitBranchIcon } from "vue-feather-icons";
import { mapState } from "vuex";
import { camelCase } from "change-case";
import { StatusBarStore } from "@/store/modules/statusBar";
import { instanceMapState } from "@/store";

export default Vue.extend({
  name: "StatusBar",
  components: {
    ChevronsRightIcon,
    GitBranchIcon,
  },
  props: {
    instanceId: { type: String },
  },
  computed: {
    applicationName(): StatusBarStore["applicationName"] {
      return instanceMapState("statusBar", this.instanceId, "applicationName");
    },
    systemName(): StatusBarStore["systemName"] {
      return instanceMapState("statusBar", this.instanceId, "systemName");
    },
    nodeName(): StatusBarStore["nodeName"] {
      return instanceMapState("statusBar", this.instanceId, "nodeName");
    },
    nodeType(): StatusBarStore["nodeType"] {
      return instanceMapState("statusBar", this.instanceId, "nodeType");
    },
    changeSetName(): StatusBarStore["changeSetName"] {
      return instanceMapState("statusBar", this.instanceId, "changeSetName");
    },
    editMode(): StatusBarStore["editMode"] {
      return instanceMapState("statusBar", this.instanceId, "editMode");
    },
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
