<template>
  <SiTabGroup>
    <template #tabs>
      <SiTabHeader>Attributes</SiTabHeader>
      <SiTabHeader>Code</SiTabHeader>
      <SiTabHeader>Resources</SiTabHeader>
    </template>

    <template #panels>
      <TabPanel class="w-full">
        <!-- FIXME(nick): remove AttributeViewer's requirement of a componentId -->
        <AttributeViewer
          :component-id="props.componentIdentification.componentId"
          :component-identification="props.componentIdentification"
          class="dark:text-neutral-50 text-neutral-900"
        />
      </TabPanel>

      <TabPanel class="w-full h-full overflow-hidden">
        <CodeViewer :code="code" class="dark:text-neutral-50 text-neutral-900">
          <template #title>
            <span
              class="text-lg ml-4 whitespace-nowrap overflow-hidden text-ellipsis"
              >{{ props.componentName }} Code</span
            >
          </template>

          <template #actionButtons>
            <SiButtonIcon
              tooltip-text="Re-generate code"
              ignore-text-color
              class="mr-4"
              :icon="currentSyncAnimate ? 'refresh-active' : 'refresh'"
              @click="generateCode"
            />
          </template>
        </CodeViewer>
      </TabPanel>

      <TabPanel class="w-full">
        <SiCollapsible
          v-for="(resource, index) in props.componentIdentification.resources"
          :key="resource.key"
          :label="resource.key"
          :default-open="index === 0"
          text-size="md"
          show-label-and-slot
        >
          <template #label>
            <HealthIcon :health="resource.health" size="md" hide-text />
          </template>
          <div class="px-xs pb-xs max-h-96 overflow-hidden flex">
            <div class="flex-grow">
              <CodeViewer :code="JSON.stringify(resource.data)" border>
                <template #title
                  ><HealthIcon :health="resource.health"
                /></template>
              </CodeViewer>
            </div>
          </div>
        </SiCollapsible>
      </TabPanel>
    </template>
  </SiTabGroup>
</template>

<script lang="ts" setup>
import { TabPanel } from "@headlessui/vue";
import { combineLatest, from, ReplaySubject, switchMap } from "rxjs";
import { fromRef, refFrom, untilUnmounted } from "vuse-rx/src";
import { computed, ref, toRefs } from "vue";
import { tag } from "rxjs-spy/operators";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import AttributeViewer from "@/organisms/AttributeViewer.vue";
import { ComponentIdentification } from "@/api/sdf/dal/component";
import CodeViewer from "@/organisms/CodeViewer.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import HealthIcon from "@/molecules/HealthIcon.vue";
import { ComponentService } from "@/service/component";
import { GlobalErrorService } from "@/service/global_error";
import { CodeView } from "@/api/sdf/dal/code_view";
import { eventCodeGenerated$ } from "@/observable/code";
import SiButtonIcon from "@/atoms/SiButtonIcon.vue";

const props = defineProps<{
  componentIdentification: ComponentIdentification;
  componentName: string;
}>();

const { componentIdentification } = toRefs(props);
const componentIdentification$ = fromRef(componentIdentification, {
  immediate: true,
});

const codeGenerated$ = new ReplaySubject<true>();
codeGenerated$.next(true); // we must fetch on setup if code gen is enabled
eventCodeGenerated$.pipe(untilUnmounted).subscribe(async (codeGenerationId) => {
  if (
    props.componentIdentification.componentId ===
    codeGenerationId?.payload.data?.componentId
  ) {
    codeGenerated$.next(true);
  }
});

const codeViews = refFrom<CodeView[]>(
  combineLatest([componentIdentification$, codeGenerated$]).pipe(
    switchMap(([componentIdentification]) => {
      return ComponentService.getCode({
        componentId: componentIdentification.componentId,
      });
    }),
    switchMap((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return from([[]]);
      } else {
        return from([response.codeViews]);
      }
    }),
    tag("codeViews"),
  ),
);

const code = computed((): string => {
  if (codeViews.value && codeViews.value.length > 0) {
    return codeViews.value[0].code ?? "# Generating code, wait a bit...";
  }
  return "# No code is better than no code! :)";
});

const currentSyncAnimate = ref<boolean>(false);

const generateCode = () => {
  currentSyncAnimate.value = true;
  ComponentService.generateCode({
    componentId: props.componentIdentification.componentId,
  }).subscribe((reply) => {
    currentSyncAnimate.value = false;
    if (reply.error) {
      GlobalErrorService.set(reply);
    } else if (!reply.success) {
      GlobalErrorService.set({
        error: {
          statusCode: 42,
          code: 42,
          message: "Code generation failed silently",
        },
      });
    }
  });
};
</script>
