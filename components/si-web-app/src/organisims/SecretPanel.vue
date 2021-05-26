<template>
  <Panel
    initialPanelType="secret"
    :panelIndex="panelIndex"
    :panelRef="panelRef"
    :panelContainerRef="panelContainerRef"
    :initialMaximizedContainer="initialMaximizedContainer"
    :initialMaximizedFull="initialMaximizedFull"
    :isVisible="isVisible"
    :isMaximizedContainerEnabled="isMaximizedContainerEnabled"
    v-on="$listeners"
  >
    <template v-slot:menuButtons>
      <div class="pl-2 align-middle">
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
      <div class="w-full">
        <SecretList v-if="showingList" />

        <div class="flex items-center justify-center" v-else-if="showingCreate">
          <div
            class="flex flex-grow px-4 py-4 mx-8 mt-8 border border-gray-700"
          >
            <SecretCreate @cancel="showList" @submit="showList" />
          </div>
        </div>
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
    panelIndex: Number,
    panelRef: String,
    panelContainerRef: String,
    initialMaximizedFull: Boolean,
    initialMaximizedContainer: Boolean,
    isVisible: Boolean,
    isMaximizedContainerEnabled: Boolean,
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
