import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { ApiRequest, addStoreHooks } from "@si/vue-lib/pinia";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { useChangeSetsStore } from "./change_sets.store";
import { UserId } from "./auth.store";
import { useRealtimeStore } from "./realtime/realtime.store";

export type LogFilters = {
  page: number;
  pageSize: number;
  sortTimestampAscending: boolean;
  changeSetFilter: ChangeSetId[];
  entityTypeFilter: string[];
  kindFilter: string[];
  userFilter: UserId[];
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
  changeSetName?: string;
  metadata: Record<string, unknown>;
}

export interface AuditLog extends AuditLogCommon {
  userName?: string;
}

export interface AuditLogDisplay extends AuditLogCommon {
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
          changeSets: [] as { id: ChangeSetId; name: string }[],
          users: [] as { id: UserId; name: string }[],
        }),
        actions: {
          async LOAD_PAGE(filters: LogFilters) {
            return new ApiRequest<{ logs: AuditLog[]; total: number }>({
              url: API_PREFIX,
              params: { ...visibility, ...filters },
              method: "get",
              onSuccess: (response) => {
                this.total = response.total;
                this.logs = response.logs.map(
                  (log) =>
                    ({
                      title: log.title,
                      userName: log.userName ?? "System",
                      userId: log.userId,
                      userEmail: log.userEmail,
                      kind: log.kind,
                      entityType: log.entityType,
                      entityName: log.entityName,
                      metadata: log.metadata,
                      timestamp: log.timestamp,
                      changeSetId: log.changeSetId,
                      changeSetName: log.changeSetName,
                    } as AuditLogDisplay),
                );
              },
            });
          },
          async GET_FILTER_OPTIONS() {
            return new ApiRequest<{
              changeSets: { id: ChangeSetId; name: string }[];
              users: { id: UserId; name: string }[];
            }>({
              url: API_PREFIX.concat(["filters"]),
              params: { ...visibility },
              method: "get",
              onSuccess: (response) => {
                // Why do we manage our own change sets and users? The admin store was the only store that had routes
                // containing this information at the time of writing, but we do not want to leak the admin store
                // outside of the admin dashbaord for security and architecture concerns. As a result, this store is,
                // at the time of writing, the only non-admin store concerned with this information.
                this.changeSets = response.changeSets;
                this.users = response.users;

                this.changeSets.sort((a, b) => a.name.localeCompare(b.name));
                this.users.sort((a, b) => a.name.localeCompare(b.name));
              },
            });
          },
        },
        onActivated() {
          this.GET_FILTER_OPTIONS();

          // TODO(nick): handle user invitations. Inviting workspace members happens in the auth portal, so need a way
          // to tell this store that it needs to add a user.
          if (workspaceId) {
            const realtimeStore = useRealtimeStore();
            realtimeStore.subscribe(this.$id, `workspace/${workspaceId}`, [
              {
                eventType: "ChangeSetCreated",
                callback: (payload, metadata) => {
                  const newChangeSet = {
                    id: metadata.change_set_id,
                    name: payload,
                  };
                  if (
                    !this.changeSets.some(
                      (changeSet) => changeSet.id === newChangeSet.id,
                    )
                  ) {
                    this.changeSets.push(newChangeSet);
                    this.changeSets.sort((a, b) =>
                      a.name.localeCompare(b.name),
                    );
                  }
                },
              },
            ]);

            return () => {
              realtimeStore.unsubscribe(this.$id);
            };
          }
        },
      },
    ),
  )();
};
