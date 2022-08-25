<template>
  <StatusBarTab :selected="selected">
    <template #icon>
      <StatusIndicatorIcon :status="tabQualificationsIconStatus" />
    </template>
    <template #name>
      <template v-if="tabQualificationsIconStatus === 'loading'">
        Running...
      </template>
      <template
        v-else-if="
          qualificationSummary?.total && qualificationSummary?.total > 0
        "
      >
        Qualifications
      </template>
      <template v-else>No Qualifications Run...</template>
    </template>
    <template v-if="qualificationSummary !== undefined" #summary>
      <StatusBarTabPill
        v-if="qualificationSummary?.total && qualificationSummary?.total > 0"
        class="border-white"
      >
        Total:
        <b class="ml-1">{{ qualificationSummary?.total }}</b>
      </StatusBarTabPill>
      <StatusBarTabPill
        v-if="
          qualificationSummary?.succeeded && qualificationSummary?.succeeded > 0
        "
        class="bg-success-100 text-success-600 font-bold"
      >
        <StatusIndicatorIcon class="w-4 text-success-600" status="success" />
        <div class="pl-px">
          {{ qualificationSummary?.succeeded }}
        </div>
      </StatusBarTabPill>

      <StatusBarTabPill
        v-if="qualificationSummary?.failed && qualificationSummary?.failed > 0"
        class="bg-destructive-100 text-destructive-600 font-bold"
      >
        <StatusIndicatorIcon
          class="w-4 text-destructive-600"
          status="failure"
        />
        <div class="pl-px">
          {{ qualificationSummary?.failed }}
        </div>
      </StatusBarTabPill>
    </template>
  </StatusBarTab>
</template>

<script lang="ts" setup>
import StatusBarTab from "@/organisms/StatusBar/StatusBarTab.vue";
import StatusIndicatorIcon from "@/molecules/StatusIndicatorIcon.vue";
import StatusBarTabPill from "@/organisms/StatusBar/StatusBarTabPill.vue";
import { QualificationService } from "@/service/qualification";
import { computed } from "vue";

defineProps<{ selected: boolean }>();

// Loads data for qualifications - total, succeeded, failed
const qualificationSummary = QualificationService.useQualificationSummary();

const tabQualificationsIconStatus = computed(() => {
  if (qualificationSummary.value === undefined) return "loading";

  const { total, succeeded, failed } = qualificationSummary.value;

  if (succeeded + failed !== total) return "loading";

  if (failed > 0) return "failure";

  return "success";
});
</script>
