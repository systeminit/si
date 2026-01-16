import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks } from "@si/vue-lib/pinia";

import { watch } from "vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { UserId, useAuthStore } from "@/store/auth.store";
import { useRealtimeStore } from "@/store/realtime/realtime.store";
import { useViewsStore } from "@/store/views.store";
import handleStoreError from "./errors";

const MOUSE_REFRESH_RATE = 5;
export const ONLINE_PING_RATE = 5000; // 5 seconds
export const ONLINE_EXPIRATION = ONLINE_PING_RATE + 1000; // 6 seconds
export const IDLE_EXPIRATION = 120000; // 2 minutes

// TODO(Wendy) - come up with user colors list, maybe talk to Mark
const COLORS = ["ffff00", "00ffff", "ff00ff", "00ff00", "0000ff", "ff0000"];

export type CursorContainerKind = "diagram" | "code-editor" | null;

export interface RawDiagramCursor {
  x: number | null;
  y: number | null;
  timestamp: Date;
  changeSetId: string | null;
  viewId: string | null;
}

export type DiagramCursorDef = RawDiagramCursor & {
  userId: UserId;
  name: string;
  color: string | undefined;
};

export interface OnlineUser {
  pk: string;
  name: string;
  pictureUrl: string | null;
  changeSetId?: string;
  viewId?: string;
  color?: string | null;
  idle: boolean;
}

export const usePresenceStore = () => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;

  const authStore = useAuthStore();
  const realtimeStore = useRealtimeStore();

  const changeSetsStore = useChangeSetsStore();

  let viewStore = useViewsStore(changeSetsStore.selectedChangeSetId);

  return addStoreHooks(
    workspaceId,
    undefined,
    defineStore(`ws${workspaceId}/presence`, {
      state: () => ({
        x: null as number | null,
        y: null as number | null,
        diagramCursorsByUserId: {} as Record<UserId, RawDiagramCursor>,
        usersById: {} as Record<UserId, OnlineUser & { lastOnlineAt: Date; lastActiveAt: Date }>,
        now: new Date(),
        lastSeenAt: new Date(),
        leftDrawerOpen: false,
        leftResizePanelWidth: 0,
        rightResizePanelWidth: 0,
      }),
      getters: {
        users(): OnlineUser[] {
          return _.values(this.usersById);
        },
        usersInChangeSet(): OnlineUser[] {
          return _.filter(this.users, (u) => u.changeSetId === changeSetsStore.selectedChangeSetId);
        },
        diagramCursors: (state): DiagramCursorDef[] => {
          return _.filter(
            _.values(
              _.mapValues(state.diagramCursorsByUserId, (cursor, userId) => ({
                ...cursor,
                userId,
                name: state.usersById[userId]?.name || "",
                color: state.usersById[userId]?.color || undefined,
              })),
            ),
            (cursor) => {
              return (
                cursor.x !== null &&
                cursor.y !== null &&
                state.usersById[cursor.userId]?.changeSetId === changeSetsStore.selectedChangeSetId &&
                state.usersById[cursor.userId]?.viewId === viewStore.selectedViewId
              );
            },
          );
        },
        isIdle: (state) => state.now.getTime() - state.lastSeenAt.getTime() > IDLE_EXPIRATION,
      },
      actions: {
        getUserColor() {
          return `#${COLORS[(this.users.length - 1) % COLORS.length]}`;
        },
        updateLastSeen() {
          this.lastSeenAt = new Date();
        },
        updateCursor(pos: { x: number; y: number } | null) {
          this.x = pos?.x || null;
          this.y = pos?.y || null;
          this.sendCursor();
        },
        clearCursor() {
          this.x = null;
          this.y = null;
          this.sendCursor();
        },
        sendOnline() {
          if (!authStore.user) return;
          realtimeStore.sendMessage({
            kind: "Online",
            data: {
              userPk: authStore.user.pk,
              name: authStore.user.name,
              pictureUrl: authStore.user.picture_url ?? null,
              idle: this.isIdle,
              changeSetId: changeSetsStore.selectedChangeSetId ?? null,
              viewId: viewStore.selectedViewId ?? null,
            },
          });
        },
        websocketSendCursor: _.debounce((x: number | null, y: number | null) => {
          if (!authStore.user) return;
          realtimeStore.sendMessage({
            kind: "Cursor",
            data: {
              userName: authStore.user.name,
              userPk: authStore.user.pk,
              changeSetId: changeSetsStore.selectedChangeSetId ?? null,
              viewId: viewStore.selectedViewId ?? null,
              container: null,
              containerKey: null,
              x: x !== null ? x.toString() : null,
              y: y !== null ? y.toString() : null,
            },
          });
        }, MOUSE_REFRESH_RATE),
        sendCursor() {
          this.websocketSendCursor(this.x, this.y);
        },
      },
      onActivated() {
        const realtimeStore = useRealtimeStore();

        this.sendCursor();
        this.sendOnline();
        const interval = setInterval(() => {
          this.sendOnline();

          // Remove users whose Online ping is too old
          this.usersById = _.pickBy(
            this.usersById,
            (user) => new Date().getTime() - user.lastOnlineAt.getTime() < ONLINE_EXPIRATION,
          );
        }, ONLINE_PING_RATE);

        watch([() => changeSetsStore.selectedChangeSetId, () => this.isIdle], () => {
          viewStore = useViewsStore(changeSetsStore.selectedChangeSetId);
          this.sendOnline();
        });

        watch([() => viewStore.selectedViewId, () => this.isIdle], this.sendOnline);

        const timeUpdate = setInterval(() => {
          this.now = new Date();
        }, 1000);

        // This subscribes to events based on your current change set for Presence data that is change set specific
        watch(
          () => changeSetsStore.selectedChangeSetId,
          (newChangeSetId) => {
            realtimeStore.unsubscribe(`${this.$id}-changeset`);
            realtimeStore.subscribe(`${this.$id}-changeset`, `changeset/${newChangeSetId}`, [
              {
                eventType: "Cursor",
                callback: (payload) => {
                  if (payload.userPk === authStore.user?.pk) return;

                  /* eslint-disable no-empty */
                  try {
                    this.diagramCursorsByUserId[payload.userPk] = {
                      changeSetId: payload.changeSetId,
                      viewId: payload.viewId,
                      x: payload.x !== null ? parseInt(payload.x) : null,
                      y: payload.y !== null ? parseInt(payload.y) : null,
                      timestamp: new Date(),
                    };
                    // Triggers watchers of cursors
                    this.diagramCursorsByUserId = {
                      ...this.diagramCursorsByUserId,
                    };
                  } catch {}
                },
              },
            ]);
          },
        );

        // This subscribes to events which are for your whole workspace
        realtimeStore.subscribe(`${this.$id}-workspace`, `workspace/${workspaceId}`, [
          {
            eventType: "Online",
            callback: (payload) => {
              if (payload.userPk === authStore.user?.pk) return;
              const needsColor = !this.usersById[payload.userPk];
              // eslint-disable-next-line @typescript-eslint/no-explicit-any
              this.usersById[payload.userPk] ||= {} as any;
              _.assign(this.usersById[payload.userPk], {
                pk: payload.userPk,
                ..._.pick(payload, "name", "idle", "pictureUrl"),
                viewId: payload.viewId,
                changeSetId: payload.changeSetId,
                lastOnlineAt: new Date(),
                ...(!payload.idle && { lastActiveAt: new Date() }),
                ...(needsColor && {
                  color: this.getUserColor(),
                }),
              });
            },
          },
        ]);

        window.addEventListener("mousedown", this.updateLastSeen);
        window.addEventListener("mousemove", this.updateLastSeen);
        window.addEventListener("keydown", this.updateLastSeen);

        const actionUnsub = this.$onAction(handleStoreError);

        return () => {
          actionUnsub();
          realtimeStore.unsubscribe(`${this.$id}-changeset`);
          realtimeStore.unsubscribe(`${this.$id}-workspace`);
          clearInterval(interval);
          clearInterval(timeUpdate);
          window.removeEventListener("mousedown", this.updateLastSeen);
          window.removeEventListener("mousemove", this.updateLastSeen);
          window.removeEventListener("keydown", this.updateLastSeen);
        };
      },
    }),
  )();
};
