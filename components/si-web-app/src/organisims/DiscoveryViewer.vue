<template>
  <div class="flex flex-col w-full" v-if="entity">
    <div
      class="flex flex-row items-center justify-between h-10 px-6 py-2 text-base text-white align-middle property-section-bg-color"
    >
      <div class="text-lg">
        {{ entity.entityType }} {{ entity.name }} discovery
      </div>
    </div>

    <div
      class="flex flex-row items-center justify-between h-10 px-8 py-2 mt-2 text-base text-white align-middle property-section-bg-color"
      v-if="implementable && hasImplItems"
    >
      <div class="text-base">
        Importable Implementations
      </div>
    </div>

    <div
      class="flex flex-row"
      v-if="
        implementable && hasImplItems && Object.keys(implementations).length > 0
      "
    >
      <div class="w-full h-full pb-2">
        <div class="flex flex-col w-full pl-4 pr-4">
          <div
            class="flex flex-col w-full py-2"
            v-for="entityType in implementationOrder"
            :key="entityType"
          >
            <div
              class="flex flex-row items-center h-8 pl-2 text-base header-background header-title"
            >
              <div>
                {{ entityType }}
              </div>
            </div>
            <div class="flex flex-col text-xs header-background">
              <div
                class="flex flex-row pl-8 mb-2"
                v-for="impl in implementations[entityType]"
                :key="impl.entity.id"
              >
                <div class="flex">
                  {{ impl.entity.name }}
                </div>
                <div class="flex pl-2">
                  <BoxIcon
                    size="1x"
                    :class="resourceStatusClass(impl.resource)"
                  />
                </div>
                <div class="flex pl-2">
                  <button
                    class="flex items-center focus:outline-none button"
                    v-if="!editMode && !changeSet"
                    @click="importImplementation(impl.entity.id)"
                  >
                    <PlusCircleIcon size="1x" class="text-sm" />
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div
      class="flex flex-row items-center justify-between h-10 px-8 py-2 mt-2 text-base text-white align-middle property-section-bg-color"
      v-if="discoverable.length > 0"
    >
      <div class="text-base">
        Discoverable
      </div>
    </div>

    <div class="flex flex-row">
      <div class="w-full h-full pb-2">
        <div
          class="flex flex-col w-full pl-4 pr-4"
          v-if="discoverable.length > 0"
        >
          <div
            class="flex flex-col w-full py-2"
            v-for="schema in discoverable"
            :key="schema.entityType"
          >
            <div
              class="flex flex-row items-center h-8 pl-2 text-base header-background header-title"
            >
              <div>
                {{ schema.entityType }}
              </div>
              <div class="flex justify-end flex-grow pr-2">
                <div class="flex pl-1">
                  <button
                    class="flex items-center focus:outline-none button"
                    v-if="!editMode"
                    @click="runSync($event, schema.entityType)"
                  >
                    <RefreshCwIcon size="1x" class="text-sm" />
                  </button>
                </div>
              </div>
            </div>
            <div class="flex flex-col text-xs header-background">
              <div
                class="flex flex-row items-center pl-8 mb-2"
                v-for="discovered in discoveredFor(schema.entityType)"
                :key="discovered.entity.id"
              >
                <div class="flex">
                  {{ discovered.entity.entityType }}
                  {{ discovered.entity.name }}
                </div>
                <div class="flex pl-2">
                  <BoxIcon
                    size="1x"
                    :class="resourceStatusClass(discovered.resource)"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
        <div class="flex flex-col items-center w-full h-full pl-2" v-else>
          <img
            v-if="!implementable"
            width="300px"
            :src="require('@/assets/images/cheech-and-chong.svg')"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { Entity } from "@/api/sdf/model/entity";
import { RegistryEntry, NodeKind, registry } from "si-registry";
import { RefreshCwIcon, PlusCircleIcon, BoxIcon } from "vue-feather-icons";
import { pluck, tap } from "rxjs/operators";
import { combineLatest } from "rxjs";
import {
  AttributeDal,
  IGetDiscoveryListReplySuccess,
} from "@/api/sdf/dal/attributeDal";
import {
  workspace$,
  resources$,
  applicationId$,
  refreshSchematic$,
  refreshEntityLabelList$,
  editMode$,
  changeSet$,
  refreshImplementations$,
} from "@/observables";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import { Resource, ResourceInternalHealth } from "si-entity";

interface Data {
  implementations: {
    [entityType: string]: {
      entity: Entity;
      resource: Resource;
    }[];
  };
  discovered: {
    [entityType: string]: {
      entity: Entity;
      resource: Resource;
    }[];
  };
}

export default Vue.extend({
  name: "DiscoveryViewer",
  props: {
    entity: {
      type: Object as PropType<Entity>,
      required: true,
    },
  },
  data(): Data {
    return {
      discovered: {},
      implementations: {},
    };
  },
  components: {
    RefreshCwIcon,
    PlusCircleIcon,
    BoxIcon,
  },
  subscriptions(): Record<string, any> {
    let entity$ = this.$watchAsObservable("entity", { immediate: true }).pipe(
      pluck("newValue"),
    );
    let discoverable$ = this.$watchAsObservable("discoverable", {
      immediate: true,
    }).pipe(pluck("newValue"));
    let fetchDiscovered$ = combineLatest(discoverable$, workspace$).pipe(
      tap(async ([discoverable, workspace]) => {
        let result: Record<string, IGetDiscoveryListReplySuccess["list"]> = {};
        if (discoverable && workspace) {
          for (const schema of discoverable) {
            let reply = await AttributeDal.getDiscoveryList({
              entityType: schema.entityType,
              workspaceId: workspace.id,
            });
            if (reply.error) {
              emitEditorErrorMessage(reply.error.message);
            } else {
              result[schema.entityType] = reply.list;
            }
          }
          // @ts-ignore
          this.discovered = result;
        } else {
          // @ts-ignore
          this.discovered = {};
        }
      }),
    );
    let fetchDiscoveredRefresh$ = combineLatest(
      discoverable$,
      workspace$,
      resources$,
    ).pipe(
      tap(async ([discoverable, workspace, resource]) => {
        if (discoverable && workspace) {
          for (const schema of discoverable) {
            if (schema.entityType == resource.entityType) {
              let reply = await AttributeDal.getDiscoveryList({
                entityType: schema.entityType,
                workspaceId: workspace.id,
              });
              if (reply.error) {
                emitEditorErrorMessage(reply.error.message);
              } else {
                // @ts-ignore
                Vue.set(this.discovered, schema.entityType, reply.list);
              }
            }
          }
        }
      }),
    );

    let implementable$ = this.$watchAsObservable("implementable", {
      immediate: true,
    }).pipe(pluck("newValue"));

    let fetchImplementations$ = combineLatest(
      applicationId$,
      implementable$,
      workspace$,
      entity$,
      refreshImplementations$,
    ).pipe(
      tap(async ([applicationId, implementable, workspace, entity]) => {
        if (applicationId && implementable && workspace && entity) {
          let implementationEntityTypes: string[] = [];
          for (const schema of Object.values(registry)) {
            if (schema.implements) {
              for (const implement of schema.implements) {
                if (implement == entity.entityType) {
                  implementationEntityTypes.push(schema.entityType);
                }
              }
            }
          }
          if (implementationEntityTypes.length > 0) {
            let reply = await AttributeDal.getImplementationsList({
              workspaceId: workspace.id,
              applicationId,
              implementationEntityTypes,
            });
            if (reply.error) {
              emitEditorErrorMessage(reply.error.message);
            } else {
              // @ts-ignore
              this.implementations = reply.list;
            }
          }
        }
      }),
    );

    return {
      fetchDiscovered: fetchDiscovered$,
      fetchDiscoveredRefresh: fetchDiscoveredRefresh$,
      fetchImplementations: fetchImplementations$,
      workspace: workspace$,
      applicationId: applicationId$,
      refreshImplementations$: refreshImplementations$,
      editMode: editMode$,
      changeSet: changeSet$,
    };
  },
  computed: {
    implementable(): boolean {
      let schema = this.entity.schema();
      let is_concept = schema.nodeKind == NodeKind.Concept;
      return is_concept;
    },
    hasImplItems(): boolean {
      let has_impl_types = this.implementationOrder.length > 0;
      let has_impl_items = false;
      for (const impl of this.implementationOrder) {
        if (this.implementations[impl].length > 0) {
          has_impl_items = true;
        }
      }
      return has_impl_types && has_impl_items;
    },
    implementationOrder(): string[] {
      return Object.keys(this.implementations).sort();
    },
    discoverable(): RegistryEntry[] {
      let discoverable: RegistryEntry[] = [];
      if (this.entity) {
        discoverable = this.entity.discoverable();
      }
      return discoverable;
    },
  },
  methods: {
    discoveredFor(entityType: string): IGetDiscoveryListReplySuccess["list"] {
      if (this.discovered && this.discovered[entityType]) {
        return this.discovered[entityType];
      } else {
        return [];
      }
    },
    async importImplementation(implementationEntityId: string) {
      // @ts-ignore
      if (this.applicationId && this.entity && this.workspace) {
        let reply = await AttributeDal.importImplementation({
          // @ts-ignore
          workspaceId: this.workspace.id,
          implementationEntityId,
          entityId: this.entity.id,
          // @ts-ignore
          applicationId: this.applicationId,
        });
        if (reply.error) {
          emitEditorErrorMessage(reply.error.message);
        }
        refreshSchematic$.next(true);
        refreshEntityLabelList$.next(true);
        refreshImplementations$.next(true);
      }
    },
    async runSync(event: MouseEvent, entityType: string) {
      if (event.target) {
        this.animateSyncButton(event.target);
      }
      // @ts-ignore
      if (this.workspace && this.entity) {
        let reply = await AttributeDal.discover({
          // @ts-ignore
          workspaceId: this.workspace.id,
          entityId: this.entity.id,
          entityType,
        });
        if (reply.error) {
          emitEditorErrorMessage(reply.error.message);
        }
      }
    },
    animateSyncButton(target: EventTarget) {
      const button = target as HTMLElement;
      if (button) {
        button.animate(
          [{ transform: "rotate(0deg)" }, { transform: "rotate(720deg)" }],
          {
            duration: 2500,
            easing: "linear",
          },
        );
      }
    },
    resourceStatusClass(r: Resource): Record<string, any> {
      let style: Record<string, any> = {};
      if (r.internalHealth == ResourceInternalHealth.Error) {
        style["error"] = true;
        return style;
      } else if (r.internalHealth == ResourceInternalHealth.Warning) {
        style["warning"] = true;
        return style;
      } else if (r.internalHealth == ResourceInternalHealth.Ok) {
        style["ok"] = true;
      } else {
        style["unknown"] = true;
      }
      return style;
    },
  },
});
</script>

<style lang="scss" scoped>
$button-saturation: 1.2;
$button-brightness: 1.1;

.property-section-bg-color {
  background-color: #292c2d;
}
.header-background {
  background-color: #1f2122;
}
.header-title {
  color: #cccdb1;
}

.button:hover {
  filter: brightness($button-brightness);
}

.button:focus {
  outline: none;
}

.button:active {
  filter: saturate(1.5) brightness($button-brightness);
}

.ok {
  color: #86f0ad;
}

.warning {
  color: #f0d286;
}

.error {
  color: #f08686;
}

.unknown {
  color: #bbbbbb;
}
</style>
