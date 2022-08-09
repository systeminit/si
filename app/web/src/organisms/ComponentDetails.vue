<template>
  <SiTabGroup>
    <template #tabs>
      <SiTabHeader>Attributes</SiTabHeader>
      <SiTabHeader>Code</SiTabHeader>
    </template>

    <template #panels>
      <TabPanel class="flex flex-col overflow-y-auto">
        <div class="text-center">
          <!-- FIXME(nick): remove AttributeViewer's requirement of a componentId -->
          <AttributeViewer
            class="dark:text-neutral-50 text-neutral-900"
            :component-id="props.componentIdentification.componentId"
            :component-identification="props.componentIdentification"
          />
        </div>
      </TabPanel>

      <TabPanel>
        <CodeViewer
          class="dark:text-neutral-50 text-neutral-900"
          :component-id="props.componentIdentification.componentId"
          :schema-name="props.componentIdentification.schemaName"
          :code="code"
          @generate="generateCode"
        >
          <template #refreshIcon>
            <RefreshIcon :class="refreshClasses" />
          </template>
        </CodeViewer>
      </TabPanel>
    </template>
  </SiTabGroup>
</template>

<script setup lang="ts">
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import { TabPanel } from "@headlessui/vue";
import AttributeViewer from "@/organisms/AttributeViewer.vue";
import _ from "lodash";
import { ComponentIdentification } from "@/api/sdf/dal/component";
import CodeViewer from "@/organisms/CodeViewer.vue";
import { ComponentService } from "@/service/component";
import { GlobalErrorService } from "@/service/global_error";
import { combineLatest, switchMap, from, ReplaySubject } from "rxjs";
import { refFrom, fromRef, untilUnmounted } from "vuse-rx/src";
import { computed, ref } from "vue";
import { CodeView } from "@/api/sdf/dal/code_view";
import { eventCodeGenerated$ } from "@/observable/code";
import { RefreshIcon } from "@heroicons/vue/solid";

const props = defineProps<{
  componentIdentification: ComponentIdentification;
}>();
const componentIdentification$ = fromRef(props.componentIdentification, {
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
  ),
);

const code = computed((): string => {
  if (codeViews.value && codeViews.value.length > 0) {
    return codeViews.value[0].code ?? "# Generating code, wait a bit...";
  }
  return "# No code is better than no code! :)";
});

const currentSyncAnimate = ref<boolean>(false);
const refreshClasses = computed(() => {
  const classes: { [key: string]: boolean } = {};
  if (currentSyncAnimate.value) {
    classes["animate-spin"] = true;
    classes["transform"] = true;
    classes["rotate-180"] = true;
  } else {
    classes["animate-spin"] = false;
    classes["transform"] = false;
    classes["rotate-180"] = false;
  }
  return classes;
});

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
