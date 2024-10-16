import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { ApiRequest, addStoreHooks } from "@si/vue-lib/pinia";
import { useWorkspacesStore, WorkspacePk } from "@/store/workspaces.store";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { ComponentId } from "@/api/sdf/dal/component";
import { useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import UpdatingModel from "../components/toasts/UpdatingModel.vue";
import handleStoreError from "./errors";
import { UserId } from "./auth.store";
import { Visibility } from "@/api/sdf/dal/visibility";
import { ActorView } from "@/api/sdf/dal/history_actor";

export enum AuditLogService {
  AuthApi = "AuthApi",
  Pinga = "Pinga",
  Rebaser = "Rebaser",
  Sdf = "Sdf",
};

export enum AuditLogKind {
  CreateComponent = "CreateComponent",
  DeleteComponent = "DeleteComponent",
  PerformedRebase = "PerformedRebase",
  RanAction = "RanAction",
  RanDependentValuesUpdate = "RanDependentValuesUpdate",
  UpdatePropertyEditorValue = "UpdatePropertyEditorValue",
};

export type LogFilters = {
  page: number;
  pageSize: number;
  sortTimestampAscending: boolean;
  excludeSystemUser: boolean;
  kindFilter: string[];
  serviceFilter: string[];
  changeSetFilter: ChangeSetId[];
  userFilter: UserId[];
};

export type AuditLog = {
  actor: ActorView,
  actorName: string;
  actorEmail: string;
  service: AuditLogService;
  kind: AuditLogKind;
  timestamp: string;
  originIpAddress: string;
  workspaceId: WorkspacePk;
  workspaceName: string;
  changeSetId: ChangeSetId;
  changeSetName: string;
}

export type AuditLogDisplay = {
  actorId: string;
  actorName: string;
  actorEmail?: string;
  service: string;
  kind: string;
  timestamp: string;
  ip: string;
  changeSetId: string;
  changeSetName: string;
};

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
      `ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/status`,
      {
        state: () => ({
          logs: [] as AuditLogDisplay[],
        }),
        getters: {},
        actions: {
          async LOAD_PAGE(filters: LogFilters) {
            return new ApiRequest<{ logs: AuditLog[] }>({
              url: API_PREFIX,
              params: { ...visibility, ...filters },
              onSuccess: (response) => {
                this.logs = response.logs.map(
                  (log: AuditLog) =>
                    ({
                      actorId: log.actor.kind,
                      actorName: log.actorName,
                      actorEmail: log.actorEmail,
                      service: log.service,
                      kind: log.kind,
                      timestamp: log.timestamp,
                      ip: log.originIpAddress,
                      changeSetId: log.changeSetId,
                      changeSetName: log.changeSetName,
                    } as AuditLogDisplay),
                );
              },
            });
          },
        },
        onActivated() {},
      },
    ),
  )();
};
