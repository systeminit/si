<template>
  <div v-if="props.componentId" class="flex flex-col w-full">
    <div
      class="flex flex-row items-center justify-between h-10 px-6 py-2 text-base text-white align-middle property-section-bg-color"
    >
      <div>
        <div>Component ID {{ props.componentId }} Resources</div>
      </div>

      <div class="flex pl-1">
        <button
          v-if="!editMode"
          ref="sync"
          class="flex items-center focus:outline-none button"
          @click="runSync()"
        >
          <VueFeather type="refresh-cw" :stroke="healthColor()" size="1.5rem" />
        </button>
        <VueFeather v-else type="box" :stroke="healthColor()" size="1.5rem" />
      </div>
    </div>

    <div class="flex flex-row">
      <div class="w-full h-full pt-2">
        <SiTextBox
          v-if="resource"
          id="resourceJson"
          name="resourceJson"
          :placeholder="JSON.stringify(resource)"
          :is-text-area="true"
          :model-value="JSON.stringify(resource)"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, defineProps } from "vue";
import { Resource, ResourceHealth } from "@/api/sdf/dal/resource";
import SiTextBox from "@/atoms/SiTextBox.vue";
import { ResourceService } from "@/service/resource";
import { GlobalErrorService } from "@/service/global_error";
import { ChangeSetService } from "@/service/change_set";
import { refFrom } from "vuse-rx";
import VueFeather from "vue-feather";

const props = defineProps<{
  componentId: number;
}>();
const editMode = refFrom<boolean>(ChangeSetService.currentEditMode());
const sync = ref<HTMLElement | null>(null);
const resource = ref<Resource | null>(null);
import { firstValueFrom } from "rxjs";

const healthColor = () => {
  if (resource.value) {
    if (resource.value.health == ResourceHealth.Ok) {
      return "#86f0ad";
    } else if (resource.value.health == ResourceHealth.Warning) {
      return "#f0d286";
    } else if (resource.value.health == ResourceHealth.Error) {
      return "#f08686";
    } else if (resource.value.health == ResourceHealth.Unknown) {
      return "#bbbbbb";
    }
  }
  return "#bbbbbb";
};

const animateSyncButton = () => {
  const button = sync.value;
  if (button) {
    button.animate(
      [{ transform: "rotate(0deg)" }, { transform: "rotate(720deg)" }],
      {
        duration: 2500,
        easing: "linear",
      },
    );
  }
};

const runSync = async () => {
  animateSyncButton();
  const reply = await firstValueFrom(
    ResourceService.syncResource({ componentId: props.componentId }),
  );
  if (reply.error) {
    GlobalErrorService.set(reply);
  } else {
    resource.value = reply.resource;
  }
};
runSync();
</script>

<style lang="scss" scoped>
$button-saturation: 1.2;
$button-brightness: 1.1;

.property-section-bg-color {
  background-color: #292c2d;
}

.header-background {
  background-color: #1f2122;
}

.button:hover {
  filter: brightness($button-brightness);
}

.button:focus {
  outline: none;
}

.button:active {
  filter: saturate(1.5) brightness($button-brightness);
}

.sync-button-invert {
  transform: scaleX(-1);
}
</style>
