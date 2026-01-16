import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { ApiRequest, addStoreHooks } from "@si/vue-lib/pinia";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { ChangeSetId, ChangeSetStatus } from "@/api/sdf/dal/change_set";
import keyedDebouncer from "@/utils/keyedDebouncer";
import { useChangeSetsStore } from "./change_sets.store";
import { UserId } from "./auth.store";
import { useRealtimeStore } from "./realtime/realtime.store";

export type AuditLogFilters = {
  changeSetFilter: string[];
  entityNameFilter: string[];
  entityTypeFilter: string[];
  titleFilter: string[];
  userFilter: string[];
};

export type AuditLogHeaderOptions = {
  changeSet: { label: string; value: string }[];
  entityName: { label: string; value: string }[];
  entityType: { label: string; value: string }[];
  title: { label: string; value: string }[];
  user: { label: string; value: string }[];
};

interface AuditLogCommon {
  title: string;
  userId?: UserId;
  userEmail?: string;
  kind: string;
  entityName: string;
  entityType: string;
  timestamp: string;
  changeSetId?: ChangeSetId;
  metadata: Record<string, unknown>;
  authenticationMethod:
    | {
        method: "System";
      }
    | {
        method: "Jwt";
        role: "Web" | "Automation";
        tokenId?: string;
      };
}

export interface AuditLog extends AuditLogCommon {
  changeSetName?: string;
  userName?: string;
}

export interface AuditLogDisplay extends AuditLogCommon {
  changeSetName: string;
  userName: string;
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

  let debouncer: ReturnType<typeof keyedDebouncer> | undefined;

  const API_PREFIX = ["v2", "workspaces", { workspaceId }, "change-sets", { selectedChangeSetId }, "audit-logs"];

  const visibility = {
    // changeSetId should not be empty if we are actually using this store
    // so we can give it a bad value and let it throw an error
    visibility_change_set_pk: changeSetId || "XXX",
  };

  return addStoreHooks(
    workspaceId,
    changeSetId,
    defineStore(`ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/audit-logs`, {
      state: () => ({
        size: 50 as number,
        sortAscending: false as boolean,
        logs: [] as AuditLogDisplay[],
        canLoadMore: true as boolean,
        filters: {
          changeSetFilter: [],
          entityNameFilter: [],
          entityTypeFilter: [],
          titleFilter: [],
          userFilter: [],
        } as AuditLogFilters,
        headerOptions: {
          changeSet: [],
          entityName: [],
          entityType: [],
          title: [],
          user: [],
        } as AuditLogHeaderOptions,
      }),
      actions: {
        async LOAD_PAGE(size: number, sortAscending: boolean, identifier?: string) {
          return new ApiRequest<{ logs: AuditLog[]; canLoadMore: boolean }>({
            url: API_PREFIX,
            params: { ...visibility, size, sortAscending },
            keyRequestStatusBy: identifier,
            method: "get",
            onSuccess: (response) => {
              this.canLoadMore = response.canLoadMore;
              this.logs = response.logs.map(
                (log) =>
                  ({
                    ...log,
                    userName: log.userName ?? "System",
                    changeSetName: log.changeSetName ?? "- none -",
                  } as AuditLogDisplay),
              );

              // TODO(nick): make everything below this comment more efficient and automatic.
              const inProgressChangeSets: Record<string, string> = {};
              const inProgressEntityNames: Record<string, string> = {};
              const inProgressEntityTypes: Record<string, string> = {};
              const inProgressTitles: Record<string, string> = {};
              const inProgressUsers: Record<string, string> = {};

              for (const log of this.logs) {
                inProgressChangeSets[log.changeSetName] = log.changeSetName;
                inProgressEntityNames[log.entityName] = log.entityName;
                inProgressEntityTypes[log.entityType] = log.entityType;
                inProgressTitles[log.title] = log.title;
                inProgressUsers[log.userName] = log.userName;
              }

              const resultForChangeSet: { label: string; value: string }[] = [];
              Object.keys(inProgressChangeSets).forEach((key) => {
                resultForChangeSet.push({ label: key, value: key });
              });
              resultForChangeSet.sort((a, b) => a.label.localeCompare(b.label));
              this.headerOptions.changeSet = resultForChangeSet;

              const resultForEntityName: { label: string; value: string }[] = [];
              Object.keys(inProgressEntityNames).forEach((key) => {
                resultForEntityName.push({ label: key, value: key });
              });
              resultForEntityName.sort((a, b) => a.label.localeCompare(b.label));
              this.headerOptions.entityName = resultForEntityName;

              const resultForEntityType: { label: string; value: string }[] = [];
              Object.keys(inProgressEntityTypes).forEach((key) => {
                resultForEntityType.push({ label: key, value: key });
              });
              resultForEntityType.sort((a, b) => a.label.localeCompare(b.label));
              this.headerOptions.entityType = resultForEntityType;

              const resultForTitle: { label: string; value: string }[] = [];
              Object.keys(inProgressTitles).forEach((key) => {
                resultForTitle.push({ label: key, value: key });
              });
              resultForTitle.sort((a, b) => a.label.localeCompare(b.label));
              this.headerOptions.title = resultForTitle;

              const resultForUser: { label: string; value: string }[] = [];
              Object.keys(inProgressUsers).forEach((key) => {
                resultForUser.push({ label: key, value: key });
              });
              resultForUser.sort((a, b) => a.label.localeCompare(b.label));
              this.headerOptions.user = resultForUser;
            },
          });
        },
        enqueueLoadPage(size: number, sortAscending: boolean, identifier: string) {
          if (!debouncer) {
            debouncer = keyedDebouncer((identifier: string) => {
              this.LOAD_PAGE(size, sortAscending, identifier);
            }, 500);
          }
          const loadPage = debouncer(identifier);
          if (loadPage) {
            loadPage(identifier);
          }
        },
      },
      onActivated() {
        if (workspaceId) {
          const realtimeStore = useRealtimeStore();
          realtimeStore.subscribe(this.$id, `workspace/${workspaceId}`, [
            {
              eventType: "AuditLogsPublished",
              callback: (payload) => {
                if (changeSetId) {
                  // If the change set of the event is the same as ours, then let's reload. Otherwise, let's only
                  // reload if we are on HEAD and the change set has been applied or abandoned.
                  if (changeSetId === payload.changeSetId) {
                    this.enqueueLoadPage(this.size, this.sortAscending, "event");
                  } else if (
                    changeSetId === changeSetsStore.headChangeSetId &&
                    (payload.changeSetStatus === ChangeSetStatus.Applied ||
                      payload.changeSetStatus === ChangeSetStatus.Abandoned)
                  ) {
                    this.enqueueLoadPage(this.size, this.sortAscending, "event");
                  }
                }
              },
            },
          ]);

          return () => {
            realtimeStore.unsubscribe(this.$id);
          };
        }
      },
    }),
  )();
};
