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
          <VueFeather type="refresh-cw" :stroke="healthColor" size="1em" />
        </button>
        <VueFeather v-else type="box" :stroke="healthColor" size="1em" />
      </div>
    </div>

    <div v-if="resource" class="flex flex-row">
      <div class="w-full h-full pt-2">
        <div class="flex flex-row mx-2 my-1">
          <div class="text-xs">
            <VueFeather type="heart" :stroke="healthColor" size="1.25em" />
          </div>

          <div class="ml-2 text-xs">
            {{ new Date(resource.updatedAt) }}
          </div>
        </div>
        <SiTextBox
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
import { ref, toRefs, computed } from "vue";
import { Resource, ResourceHealth } from "@/api/sdf/dal/resource";
import SiTextBox from "@/atoms/SiTextBox.vue";
import { ComponentService } from "@/service/component";
import { GlobalErrorService } from "@/service/global_error";
import { ChangeSetService } from "@/service/change_set";
import { fromRef, refFrom } from "vuse-rx";
import VueFeather from "vue-feather";
import { system$ } from "@/observable/system";
import { from, combineLatest, ReplaySubject } from "rxjs";
import { switchMap } from "rxjs/operators";
import { eventResourceSynced$ } from "@/observable/resource";

const props = defineProps<{
  componentId: number;
}>();
const { componentId } = toRefs(props);

const editMode = refFrom<boolean>(ChangeSetService.currentEditMode());
const sync = ref<HTMLElement | null>(null);

// NOTE(nick): making this "computed" will result in the active view breaking for the attribute panel.
const healthColor = computed(() => {
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
});

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

const runSync = () => {
  animateSyncButton();
  ComponentService.syncResource({ componentId: props.componentId }).subscribe(
    (reply) => {
      if (reply.error) {
        GlobalErrorService.set(reply);
      } else if (!reply.success) {
        GlobalErrorService.set({
          error: {
            statusCode: 42,
            code: 42,
            message: "Sync failed silently",
          },
        });
      }
    },
  );
};

const resourceSynced$ = new ReplaySubject<true>();
resourceSynced$.next(true); // We must fetch on setup
eventResourceSynced$.subscribe((resourceSyncId) => {
  combineLatest([system$]).pipe(
    switchMap(([system]) => {
      const data = resourceSyncId?.payload.data;
      const sameComponent = props.componentId === data?.componentId;
      const sameSystem = system?.id === data?.systemId;
      if (sameComponent && sameSystem) {
        resourceSynced$.next(true);
      }
    }),
  );
});

// Fetches the resource. First by listening to componentId$.
// If it emits a value, we are re run the pipeline that follows.
//
// The pipeline starts with calling the getResource service, and switchMap-ing
// to the result of that observable. (So now we are emitting a value every time
// this observable emits)
//
// We then take the emitted value from that observable, which is the reply,
// check it for errors (if there are errors, set the resource to null). Otherwise
// we set the resource to the returned value, and we're done.
const resource = refFrom<Resource | null>(
  combineLatest([componentId$, resourceSynced$]).pipe(
    switchMap(([componentId]) => {
      if (componentId) {
        return ComponentService.getResource({ componentId });
      } else {
        return from([null]);
      }
    }),
    switchMap((reply) => {
      if (reply === null) {
        return from([null]);
      } else if (reply.error) {
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
