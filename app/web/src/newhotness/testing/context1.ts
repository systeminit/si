import { computed, ref } from "vue";
import { AttributeInputContext, Context } from "@/newhotness/types";
import { AttributeErrors } from "@/newhotness/AttributePanel.vue";
import { ChangeSetStatus } from "@/api/sdf/dal/change_set";

export const ATTRIBUTEINPUT: AttributeInputContext = { blankInput: false };
export const ATTRIBUTE_ERRORS = computed<AttributeErrors>(() => {
  return {
    saveErrors: ref({}),
  };
});
export const CONTEXT = computed<Context>(() => {
  return {
    workspacePk: computed(() => "01HRFEV0S23R1G23RP75QQDCA7"),
    changeSetId: computed(() => "01K45ZAY3PQPJ457V65KNCC66F"),
    changeSet: ref({
      baseChangeSetId: "01JYPTEC5JM3T1Y4ECEPT9560J",
      createdAt: "2025-09-02T19:44:20.609624Z" as IsoDateString,
      id: "01K45ZAY3PQPJ457V65KNCC66F",
      name: "test",
      status: "Open" as ChangeSetStatus,
      updatedAt: "2025-09-08T21:11:45.779873Z" as IsoDateString,
      workspaceId: "01HRFEV0S23R1G23RP75QQDCA7",
    }),
    approvers: ref([]),
    user: {
      created_at: "2025-06-26T19:11:44.656758Z",
      email: "",
      name: "",
      picture_url: "",
      pk: "01HRFEV0RMWMH5SGBGDARH3G48",
      updated_at: "2025-06-26T19:11:44.656758Z",
    },
    userWorkspaceFlags: ref({}),
    onHead: computed(() => {
      return false;
    }),
    headChangeSetId: ref("01JYPTEC5JM3T1Y4ECEPT9560J"),
    outgoingCounts: computed(() => ({})),
    componentDetails: computed(() => ({})),
    schemaMembers: computed(() => ({})),
    queriesEnabled: ref(true),
    reopenOnboarding: () => false,
  };
});
