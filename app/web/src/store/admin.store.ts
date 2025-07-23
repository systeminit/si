import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { UserId, WorkspaceUser } from "@/store/auth.store";
import { FuncRunId } from "@/store/func_runs.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useRealtimeStore } from "@/store/realtime/realtime.store";
import { ChangeSetId, ChangeSetStatus } from "@/api/sdf/dal/change_set";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  AttributePrototypeArgumentId,
  FuncArgumentId,
  FuncId,
} from "@/api/sdf/dal/func";
import { PropId } from "@/api/sdf/dal/prop";
import { InputSocketId, OutputSocketId } from "@/api/sdf/dal/schema";
import { WorkspacePk } from "@/api/sdf/dal/workspace";
import { AttributeValueId } from "./status.store";
import { WsEventPayloadMap } from "./realtime/realtime_events";

export interface AdminWorkspace {
  id: WorkspacePk;
  name: string;
  defaultChangeSetId: ChangeSetId;
  componentConcurrencyLimit?: number;
  snapshotVersion: string;
}

export interface AdminChangeSet {
  id: ChangeSetId;
  name: string;
  status: ChangeSetStatus;
  baseChangeSetId?: ChangeSetId;
  workspaceSnapshotAddress: string;
  workspaceId: WorkspacePk;
  mergeRequestedByUserId?: UserId;
}

export interface ConnectionMigrationRun {
  workspaceId: WorkspacePk;
  changeSetId: ChangeSetId;
  startedAt: Date;
  dryRun: boolean;
  migrations: ConnectionMigration[];
  summary?: WsEventPayloadMap["ConnectionMigrationFinished"] & {
    finishedAt: Date;
  };
}

export const useAdminStore = () => {
  const API_PREFIX = `v2/admin`;
  const WORKSPACE_API_PREFIX = ["v2", "workspaces"];

  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;
  const realtimeStore = useRealtimeStore();

  return addStoreHooks(
    workspaceId,
    null,
    defineStore(`ws${workspaceId || "NONE"}/admin`, {
      state: () => ({
        updatingModuleCacheOperationId: null as string | null,
        updatingModuleCacheOperationError: undefined as string | undefined,
        updatingModuleCacheOperationRunning: false as boolean,
        clearingInnitCacheOperationRunning: false as boolean,
        validateSnapshotResponse: undefined as
          | ValidateSnapshotResponse
          | undefined,
        connectionMigrationRun: undefined as ConnectionMigrationRun | undefined,
      }),
      getters: {},
      actions: {
        async SEARCH_WORKSPACES(query?: string) {
          return new ApiRequest<
            {
              workspaces: AdminWorkspace[];
            },
            { query?: string }
          >({
            method: "get",
            params: { query },
            url: `${API_PREFIX}/workspaces`,
          });
        },
        async LIST_CHANGE_SETS(workspaceId: string) {
          return new ApiRequest<{
            changeSets: { [key: string]: AdminChangeSet };
          }>({
            keyRequestStatusBy: workspaceId,
            method: "get",
            url: `${API_PREFIX}/workspaces/${workspaceId}/change_sets`,
          });
        },
        // TODO(nick): remove in favor of the auth store call.
        async LIST_WORKSPACE_USERS(workspaceId: string) {
          return new ApiRequest<{ users: WorkspaceUser[] }>({
            method: "get",
            url: WORKSPACE_API_PREFIX.concat([workspaceId, "users"]),
          });
        },
        async GET_SNAPSHOT(workspaceId: string, changeSetId: string) {
          return new ApiRequest<string>({
            method: "get",
            url: `${API_PREFIX}/workspaces/${workspaceId}/change_sets/${changeSetId}/get_snapshot`,
          });
        },
        async SET_SNAPSHOT(
          workspaceId: string,
          changeSetId: string,
          snapshot: Blob,
        ) {
          const formData = new FormData();
          formData.append("snapshot", snapshot);

          return new ApiRequest<{ workspaceSnapshotAddress: string }>({
            method: "post",
            url: `${API_PREFIX}/workspaces/${workspaceId}/change_sets/${changeSetId}/set_snapshot`,
            formData,
          });
        },
        async GET_CAS_DATA(workspaceId: string, changeSetId: string) {
          return new ApiRequest<string>({
            method: "get",
            url: `${API_PREFIX}/workspaces/${workspaceId}/change_sets/${changeSetId}/get_cas_data`,
          });
        },
        async UPLOAD_CAS_DATA(
          workspaceId: string,
          changeSetId: string,
          casData: Blob,
        ) {
          const formData = new FormData();
          formData.append("cas_data", casData);

          return new ApiRequest<{ workspaceSnapshotAddress: string }>({
            method: "post",
            url: `${API_PREFIX}/workspaces/${workspaceId}/change_sets/${changeSetId}/upload_cas_data`,
            formData,
          });
        },
        async SET_CONCURRENCY_LIMIT(
          workspaceId: string,
          concurrencyLimit?: number,
        ) {
          return new ApiRequest<
            { concurrencyLimit?: number },
            { concurrencyLimit?: number }
          >({
            method: "post",
            url: `${API_PREFIX}/workspaces/${workspaceId}/set_concurrency_limit`,
            params: { concurrencyLimit },
          });
        },
        async KILL_EXECUTION(funcRunId: FuncRunId) {
          return new ApiRequest<null>({
            method: "put",
            url: `${API_PREFIX}/func/runs/${funcRunId}/kill_execution`,
          });
        },
        async UPDATE_MODULE_CACHE() {
          this.updatingModuleCacheOperationRunning = true;
          this.updatingModuleCacheOperationId = null;
          this.updatingModuleCacheOperationError = undefined;

          return new ApiRequest<{ id: string }>({
            method: "post",
            url: `${API_PREFIX}/update_module_cache`,
            onSuccess: (response) => {
              this.updatingModuleCacheOperationId = response.id;
            },
            onFail: () => {
              this.updatingModuleCacheOperationRunning = false;
            },
          });
        },
        async CLEAR_INNIT_CACHE() {
          this.clearingInnitCacheOperationRunning = true;

          return new ApiRequest<{ id: string }>({
            method: "post",
            url: `${API_PREFIX}/innit/cache/clear`,
            onSuccess: (response) => {
              this.clearingInnitCacheOperationRunning = false;
            },
            onFail: () => {
              this.clearingInnitCacheOperationRunning = false;
            },
          });
        },
        async MIGRATE_CONNECTIONS(
          workspaceId: string,
          changeSetId: string,
          options?: { dryRun?: boolean },
        ) {
          return new ApiRequest<unknown>({
            method: options?.dryRun ? "get" : "post",
            url: `v2/admin/workspaces/${workspaceId}/change_sets/${changeSetId}/migrate_connections`,
          });
        },
        async VALIDATE_SNAPSHOT(
          workspaceId: string,
          changeSetId: string,
          options?: { fixIssues?: boolean },
        ) {
          return new ApiRequest<ValidateSnapshotResponse>({
            method: options?.fixIssues ? "post" : "get",
            url: `v2/admin/workspaces/${workspaceId}/change_sets/${changeSetId}/validate_snapshot`,
            onSuccess: (response) => {
              this.validateSnapshotResponse = response;
            },
          });
        },
      },
      onActivated() {
        realtimeStore.subscribe(this.$id, `workspace/${workspaceId}`, [
          {
            eventType: "AsyncFinish",
            callback: async ({ id }) => {
              if (id === this.updatingModuleCacheOperationId) {
                this.updatingModuleCacheOperationRunning = false;
              }
            },
          },
          {
            eventType: "AsyncError",
            callback: async ({ id, error }) => {
              if (id === this.updatingModuleCacheOperationId) {
                this.updatingModuleCacheOperationError = error;
                this.updatingModuleCacheOperationRunning = false;
              }
            },
          },
          {
            eventType: "ConnectionMigrationStarted",
            callback: async ({ dryRun }, { workspace_pk, change_set_id }) => {
              this.connectionMigrationRun = {
                workspaceId: workspace_pk,
                changeSetId: change_set_id,
                dryRun,
                startedAt: new Date(),
                migrations: [],
              };
            },
          },
          {
            eventType: "ConnectionMigrated",
            callback: async (migration) => {
              if (this.connectionMigrationRun) {
                this.connectionMigrationRun.migrations.push(migration);
              }
            },
          },
          {
            eventType: "ConnectionMigrationFinished",
            callback: async (summary) => {
              if (this.connectionMigrationRun) {
                this.connectionMigrationRun.summary = {
                  ...summary,
                  finishedAt: new Date(),
                };
              }
            },
          },
        ]);
        return () => {};
      },
    }),
  )();
};

export interface ValidateSnapshotResponse {
  issues: ValidationIssue[];
}

export type ValidationIssue =
  | {
      type: "connectionToUnknownSocket";
      dest_apa: AttributePrototypeArgumentId;
      source_component: ComponentId;
      source_socket: OutputSocketId;
      message: string;
      fixed: boolean;
    }
  | {
      type: "duplicateAttributeValue";
      original: AttributeValueId;
      duplicate: AttributeValueId;
      message: string;
      fixed: boolean;
    }
  | {
      type: "duplicateAttributeValueWithDifferentValues";
      original: AttributeValueId;
      duplicate: AttributeValueId;
      message: string;
      fixed: boolean;
    }
  | {
      type: "missingChildAttributeValues";
      object: AttributeValueId;
      missing_children: PropId[];
      message: string;
      fixed: boolean;
    }
  | {
      type: "unknownChildAttributeValue";
      child: AttributeValueId;
      message: string;
      fixed: boolean;
    };

export type ConnectionMigration =
  // If there is no issue, both socket and prop connections are always defined.
  | {
      issue?: undefined;
      explicitConnectionId?: AttributePrototypeArgumentId;
      socketConnection: ConnectionMigrationSocketConnection;
      propConnections: ConnectionMigrationPropConnection[];
      message: string;
      migrated: boolean;
    }
  // If there is an issue with an explicit connection, socket connections may be undefined
  // (because we may have identified the connection but been unable to completely build them out).
  | {
      issue: ConnectionUnmigrateableBecause;
      explicitConnectionId: AttributePrototypeArgumentId;
      socketConnection?: ConnectionMigrationSocketConnection;
      propConnections: ConnectionMigrationPropConnection[];
      message: string;
      migrated: boolean;
    }
  // If there is an issue with an inferred connection, socketConnection is still always defined.
  | {
      issue: ConnectionUnmigrateableBecause;
      explicitConnectionId?: undefined;
      socketConnection: ConnectionMigrationSocketConnection;
      propConnections: ConnectionMigrationPropConnection[];
      message: string;
      migrated: boolean;
    };

export interface ConnectionMigrationSocketConnection {
  source: { componentId: ComponentId; socketId: OutputSocketId };
  destination: { componentId: ComponentId; socketId: InputSocketId };
}

export interface ConnectionMigrationPropConnection {
  destAvId: AttributeValueId;
  sourceRootAvId: AttributeValueId;
  sourcePath: string;
  funcId: FuncId;
  funcArgId: FuncArgumentId;
}

export type ConnectionUnmigrateableBecause =
  | { type: "connectionPrototypeHasMultipleArgs" }
  | { type: "destinationIsNotInputSocket" }
  | { type: "destinationSocketArgumentNotBoundToProp" }
  | { type: "destinationSocketBoundToPropWithNoValue"; destPropId: PropId }
  | { type: "destinationSocketHasMultipleBindings" }
  | { type: "destinationSocketHasNoBindings" }
  | { type: "invalidGraph" }
  | { type: "multipleConnectionsToSameSocket" }
  | { type: "noArgumentTargets" }
  | {
      type: "sourceAndDestinationSocketBothHaveFuncs";
      sourceFuncId: FuncId;
      destFuncId: FuncId;
    }
  | { type: "sourceSocketPrototypeArgumentNotBoundToProp" }
  | { type: "sourceSocketPrototypeHasMultipleArguments" }
  | { type: "sourceSocketPrototypeHasNoArguments" };
