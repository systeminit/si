import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { ApiRequest, addStoreHooks } from "@si/vue-lib/pinia";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { useChangeSetsStore } from "./change_sets.store";
import { UserId } from "./auth.store";

export type LogFilters = {
  page: number;
  pageSize: number;
  sortTimestampAscending: boolean;
  excludeSystemUser: boolean;
  kindFilter: string[];
  changeSetFilter: ChangeSetId[];
  userFilter: UserId[];
};

interface AuditLogCommon {
  displayName: string;
  userId?: UserId;
  userEmail?: string;
  kind: string;
  entityType: string;
  timestamp: string;
  changeSetId?: ChangeSetId;
  changeSetName?: string;
  metadata: Record<string, unknown>;
}

export interface AuditLog extends AuditLogCommon {
  userName?: string;
  entityName?: string;
}

export interface AuditLogDisplay extends AuditLogCommon {
  userName: string;
  entityName: string;
}

export const useLogsStore = (forceChangeSetId?: ChangeSetId) => {
  // this needs some work... but we'll probably want a way to force using HEAD
  // so we can load HEAD data in some scenarios while also loading a change set?
  let changeSetId: ChangeSetId | undefined;
  if (forceChangeSetId) {
    changeSetId = forceChangeSetId;
  } else {
    const changeSetsStore = useChangeSetsStore();
    changeSetId = changeSetsStore.selectedChangeSetId;
  }

  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;
  const changeSetsStore = useChangeSetsStore();
  const selectedChangeSetId = changeSetsStore.selectedChangeSet?.id;

  const API_PREFIX = [
    "v2",
    "workspaces",
    { workspaceId },
    "change-sets",
    { selectedChangeSetId },
    "audit-logs",
  ];

  const visibility = {
    // changeSetId should not be empty if we are actually using this store
    // so we can give it a bad value and let it throw an error
    visibility_change_set_pk: changeSetId || "XXX",
  };

  return addStoreHooks(
    workspaceId,
    changeSetId,
    defineStore(
      `ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/audit-logs`,
      {
        state: () => ({
          logs: [] as AuditLogDisplay[],
          total: 0 as number,
        }),
        getters: {},
        actions: {
          async LOAD_PAGE(filters: LogFilters) {
            return new ApiRequest<{ logs: AuditLog[]; total: number }>({
              url: API_PREFIX,
              params: { ...visibility, ...filters },
              onSuccess: (response) => {
                this.logs = response.logs.map(
                  (log: AuditLog) =>
                    ({
                      displayName: log.displayName,
                      userName: log.userName ?? "System",
                      userId: log.userId,
                      userEmail: log.userEmail,
                      kind: log.kind,
                      entityType: log.entityType,
                      entityName: log.entityName ?? "-",
                      metadata: log.metadata,
                      timestamp: log.timestamp,
                      changeSetId: log.changeSetId,
                      changeSetName: log.changeSetName,
                    } as AuditLogDisplay),
                );
                this.total = response.total;
              },
            });
          },
        },
        onActivated() {},
      },
    ),
  )();
};
