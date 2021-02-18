<template>
  <Panel
    initialPanelType="secret"
    :panelRef="panelRef"
    :panelContainerRef="panelContainerRef"
    :initialMaximizedContainer="initialMaximizedContainer"
    :initialMaximizedFull="initialMaximizedFull"
    v-on="$listeners"
  >
    <template v-slot:menuButtons>
      <div class="align-middle pl-2">
        <SiButton
          icon="plus"
          label="New"
          size="xs"
          :disabled="showingCreate"
          @click.native="showCreate"
        />
      </div>
    </template>
    <template v-slot:content>
      <div>
        <SecretList v-if="showingList" />
        <SecretCreate
          v-else-if="showingCreate"
          @cancel="showList"
          @submit="showList"
        />
      </div>
    </template>
  </Panel>
</template>

<script lang="ts">
import Vue from "vue";
import Panel from "@/molecules/Panel.vue";
import SecretCreate from "@/organisims/SecretCreate.vue";
import SecretList from "@/organisims/SecretList.vue";
import SiButton from "@/atoms/SiButton.vue";

interface IData {
  showing: "list" | "create";
}

export default Vue.extend({
  name: "SecretPanel",
  components: {
    Panel,
    SecretCreate,
    SecretList,
    SiButton,
  },
  props: {
    panelRef: String,
    panelContainerRef: String,
    initialMaximizedFull: Boolean,
    initialMaximizedContainer: Boolean,
  },
  data(): IData {
    return {
      showing: "list",
    };
  },
  computed: {
    showingList(): boolean {
      return this.showing == "list";
    },
    showingCreate(): boolean {
      return this.showing == "create";
    },
  },
  methods: {
    showList() {
      this.showing = "list";
    },
    showCreate() {
      this.showing = "create";
    },
  },
});
</script>
