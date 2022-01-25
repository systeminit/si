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
import { ref, defineProps, toRefs } from "vue";
import { Resource, ResourceHealth } from "@/api/sdf/dal/resource";
import SiTextBox from "@/atoms/SiTextBox.vue";
import { ResourceService } from "@/service/resource";
import { GlobalErrorService } from "@/service/global_error";
import { ChangeSetService } from "@/service/change_set";
import { fromRef, refFrom } from "vuse-rx";
import VueFeather from "vue-feather";
import { combineLatest, from, ReplaySubject, switchMap } from "rxjs";

const props = defineProps<{
  componentId: number;
}>();
const { componentId } = toRefs(props);

const editMode = refFrom<boolean>(ChangeSetService.currentEditMode());
const sync = ref<HTMLElement | null>(null);

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

// We need an observable stream of props.componentId. We also want
// that stream to emit a value immediately (the first value, as well as all
// subsequent values)
const componentId$ = fromRef<number>(componentId, { immediate: true });

// Then we need a replay subject that just emits true values. This is the
// trigger for the 'sync' button to reload the data.
const runSync$ = new ReplaySubject<true>(1);

// We want it to be 'hot', meaning that any time this observable is subscribed
// to, it has a value. So we prime it with `true`.
runSync$.next(true);

// When the user clicks the sync button, we emit a new `true` value.
const runSync = () => {
  runSync$.next(true);
};

// Compute the actual resource. First by listening to the two trigger
// observables - the componentId$ and the runSync$. If either of those
// emit a value, we are going to re run the pipeline that follows.
//
// The pipeline starts with calling the syncResource service, and switchMap-ing
// to the result of that observable. (So now we are emitting a value every time
// this observable emits)
//
// We then take the emitted value from that observable, which is the reply,
// check it for errors (if there are errors, set the resource to null). Otherwise
// we set the resource to the returned value, and we're done.
const resource = refFrom<Resource | null>(
  combineLatest([componentId$, runSync$]).pipe(
    switchMap(([componentId]) => {
      animateSyncButton();
      return ResourceService.syncResource({ componentId });
    }),
    switchMap((reply) => {
      if (reply.error) {
        GlobalErrorService.set(reply);
        return from([null]);
      } else {
        return from([reply.resource]);
      }
    }),
  ),
);
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
