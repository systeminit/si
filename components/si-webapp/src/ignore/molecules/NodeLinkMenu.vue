<template>
  <div
    class="relative w-auto menu-root"
    @mouseleave="onMouseLeave"
    @mouseenter="cancelClose"
  >
    <button
      @click="isOpen = !isOpen"
      class="w-full focus:outline-none"
      :disabled="isDisabled"
      data-cy="editor-schematic-node-add-button"
    >
      <div
        class="items-center self-center w-full text-sm subpixel-antialiased font-light tracking-tight"
        :class="{
          'text-gray-200': !isOpen,
          'menu-selected': isOpen,
          'text-gray-600': isDisabled,
        }"
      >
        Link
      </div>
    </button>

    <NodeLinkMenuCategory
      :isOpen="isOpen"
      :menuItems="menuItems"
      rootMenu
      class="menu-root"
      @selected="onSelect"
    />
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import _ from "lodash";

import {
  entityMenu,
  entityTypesForMenu,
  MenuItem,
  LinkNodeItem,
} from "si-registry";
import NodeLinkMenuCategory from "./NodeLinkMenu/NodeLinkMenuCategory.vue";
import { Workspace } from "@/api/sdf/model/workspace";
import { ChangeSet } from "@/api/sdf/model/changeSet";
import { EditSession } from "@/api/sdf/model/editSession";
import { pluck, switchMap, tap } from "rxjs/operators";
import { Observable, combineLatest, of, from } from "rxjs";
import { workspace$, changeSet$, editSession$ } from "@/observables";
import {
  IGetNodeLinkMenuRequest,
  SchematicDal,
  IGetNodeLinkMenuReplySuccess,
} from "@/api/sdf/dal/schematicDal";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";

interface Data {
  isOpen: boolean;
}

export interface AddMenuSelectedPayload {
  entityType: string;
  event: MouseEvent;
}

const debounceIsOpen = _.debounce((component: any, isOpen: boolean) => {
  component.isOpen = isOpen;
}, 500);

export default Vue.extend({
  name: "NodeLinkMenu",
  props: {
    positionCtx: {
      type: String,
    },
    disabled: {
      type: Boolean,
      default: false,
    },
    filter: {
      type: Object as PropType<Parameters<typeof entityMenu>[0]>,
    },
  },
  components: {
    NodeLinkMenuCategory,
  },
  data(): Data {
    return {
      isOpen: false,
    };
  },
  computed: {
    isDisabled(): boolean {
      if (this.disabled) {
        return true;
      } else {
        // @ts-ignore
        if (this.menuItems && this.menuItems.length) {
          return false;
        } else {
          return true;
        }
      }
    },
  },
  subscriptions(): Record<string, any> {
    const positionCtx$: Observable<string> = this.$watchAsObservable<string>(
      () =>
        // @ts-ignore
        this.positionCtx,
      {
        immediate: true,
      },
    ).pipe(pluck("newValue"));
    const filter$: Observable<Parameters<
      typeof entityMenu
    >[0]> = this.$watchAsObservable<Parameters<typeof entityMenu>[0]>(
      () =>
        // @ts-ignore
        this.filter,
      {
        immediate: true,
      },
    ).pipe(pluck("newValue"));

    const menuItems$: Observable<MenuItem[]> = combineLatest(
      filter$,
      positionCtx$,
      workspace$,
      changeSet$,
      editSession$,
    ).pipe(
      switchMap(([filter, positionCtx, workspace, changeSet, editSession]) => {
        if (filter && positionCtx && workspace && changeSet && editSession) {
          const entityTypes = entityTypesForMenu(filter);
          const request: IGetNodeLinkMenuRequest = {
            entityTypes,
            workspaceId: workspace.id,
            changeSetId: changeSet.id,
            editSessionId: editSession.id,
            positionCtx,
          };
          return from(SchematicDal.getNodeLinkMenu(request));
        } else {
          let defaultData: IGetNodeLinkMenuReplySuccess = {
            link: [],
          };
          return from([defaultData]);
        }
      }),
      tap(result => {
        if (result.error) {
          emitEditorErrorMessage(result.error.message);
        }
      }),
      switchMap(reply => {
        if (reply.link) {
          // @ts-ignore
          const menu = entityMenu(this.filter, reply.link).list;
          return of(menu);
        } else {
          return of([]);
        }
      }),
    );

    return {
      menuItems: menuItems$,
    };
  },
  methods: {
    onSelect(toLink: LinkNodeItem, event: MouseEvent): void {
      event.preventDefault();
      this.$emit("selected", toLink, event);
      this.isOpen = false;
    },
    onMouseLeave(): void {
      debounceIsOpen(this, false);
    },
    cancelClose(): void {
      debounceIsOpen.cancel();
    },
  },
  created() {
    const handleEscape = (e: any) => {
      if (e.key === "Esc" || e.key === "Escape") {
        if (this.isOpen) {
          this.isOpen = false;
        }
      }
    };
    document.addEventListener("keydown", handleEscape);
    this.$once("hook:beforeDestroy", () => {
      document.removeEventListener("keydown", handleEscape);
    });
  },
});
</script>

<style>
.menu-root {
  z-index: 999;
}

.menu {
  background-color: #2d3748;
  border-color: #485359;
}

.menu-selected {
  background-color: #edf2f8;
  color: #000000;
}

.menu-not-selected {
  color: red;
}

.options {
  background-color: #1f2631;
  border-color: #485359;
}
.options:hover {
  background-color: #3d4b62;
  border-color: #454d3e;
}

.menu-category .category-items {
  /*  visibility: hidden; */
  position: absolute;
  left: 100%;
  top: auto;
  margin-top: -1.305rem;
}

.menu-category:hover .category-items {
  /* visibility: visible; */
}
</style>
