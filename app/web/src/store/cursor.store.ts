import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks } from "@si/vue-lib/pinia";

import { DiagramCursorDef } from "@/components/GenericDiagram/diagram_types";
import { ChangeSetId, useChangeSetsStore } from "@/store/change_sets.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useAuthStore } from "@/store/auth.store";
import { useRealtimeStore } from "@/store/realtime/realtime.store";

export type UserId = string;

const MOUSE_REFRESH_RATE = 20;
const MOUSE_EXPIRATION = 5000;

export type CursorContainerKind = "diagram" | "code-editor" | null;

export const useCursorStore = (forceChangeSetId?: ChangeSetId) => {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;

  const authStore = useAuthStore();
  const realtimeStore = useRealtimeStore();

  const changeSetsStore = useChangeSetsStore();
  // this needs some work... but we'll probably want a way to force using HEAD
  // so we can load HEAD data in some scenarios while also loading a change set?
  let changeSetId: ChangeSetId | null;
  if (forceChangeSetId) {
    changeSetId = forceChangeSetId;
  } else {
    changeSetId = changeSetsStore.selectedChangeSetId;
  }

  return addStoreHooks(
    defineStore(
      `ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/cursor`,
      {
        state: () => ({
          x: null as number | null,
          y: null as number | null,
          container: null as CursorContainerKind,
          containerKey: null as string | null,
          cursors: {} as Record<UserId, DiagramCursorDef>,
        }),
        actions: {
          updateCursor(x: number, y: number) {
            this.x = x;
            this.y = y;
            this.send();
          },
          setContainer(container: CursorContainerKind, key: string | null) {
            if (this.container === container) return;
            this.x = 0;
            this.y = 0;
            this.container = container;
            this.containerKey = key;
            this.cursors = {};
            this.send();
          },
          cleanupCursor() {
            const toDelete = [];
            for (const key in this.cursors) {
              const cursor = this.cursors[key];
              if (
                cursor &&
                new Date().getTime() - cursor.timestamp.getTime() >
                  MOUSE_EXPIRATION
              ) {
                toDelete.push(key);
              }
            }
            for (const key of toDelete) {
              delete this.cursors[key];
            }
          },
          websocketSendCursor: _.debounce(
            (
              container: CursorContainerKind,
              containerKey: string | null,
              x: number,
              y: number,
            ) => {
              if (!authStore.user) return;
              realtimeStore.sendMessage({
                kind: "Cursor",
                data: {
                  userName: authStore.user.name,
                  userPk: authStore.user.pk,
                  changeSetPk: changeSetsStore.selectedChangeSetId,
                  containerKey,
                  container,
                  x: `${x}`,
                  y: `${y}`,
                },
              });
            },
            MOUSE_REFRESH_RATE,
          ),
          send() {
            this.cleanupCursor();

            if (this.x === null || this.y === null) return;

            this.websocketSendCursor(
              this.container,
              this.containerKey,
              this.x,
              this.y,
            );

            // Avoids sending the cursor twice if outside of the diagram for now
            if (this.container === null) {
              this.x = null;
              this.y = null;
            }
          },
        },
        onActivated() {
          const interval = setInterval(this.send, MOUSE_EXPIRATION - 500);

          const realtimeStore = useRealtimeStore();
          realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
            {
              eventType: "Cursor",
              callback: (payload) => {
                if (payload.userPk === authStore.user?.pk) return;
                if (
                  payload.container !== this.container ||
                  payload.containerKey !== this.containerKey
                ) {
                  delete this.cursors[payload.userPk];
                } else {
                  /* eslint-disable no-empty */
                  try {
                    this.cursors[payload.userPk] = {
                      x: parseInt(payload.x),
                      y: parseInt(payload.y),
                      userPk: payload.userPk,
                      userName: payload.userName,
                      timestamp: new Date(),
                    };
                    // Triggers watchers of cursors
                    this.cursors = { ...this.cursors };
                  } catch {}
                }
              },
            },
          ]);

          return () => {
            realtimeStore.unsubscribe(this.$id);
            clearInterval(interval);
          };
        },
      },
    ),
  )();
};
