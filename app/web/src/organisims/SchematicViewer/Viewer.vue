<template>
  <div :id="container.id" :ref="container.element" class="w-full h-full">
    <div v-if="debug" class="flex flex-row">
      <div class="ml-2 font-medium text-yellow-200">{{ state.value }}</div>
    </div>
    <canvas
      :id="canvas.id"
      :ref="canvas.element"
      @wheel="handleMouseWheel"
      @mouseenter="mouseEnter()"
      @mouseleave="mouseLeave()"
    />
  </div>
</template>

<script setup lang="ts">
import { watch, onMounted, onUnmounted, ref, toRefs } from "vue";
import _ from "lodash";
import * as Rx from "rxjs";
import { Node } from "./Viewer/obj/node";
import * as SM from "./state_machine";
import { untilUnmounted } from "vuse-rx";
import { Interpreter } from "xstate";

import { useMachine } from "@xstate/vue";
import { componentsMetadata$ } from "@/observable/component";
import { ComponentMetadata } from "@/service/component/get_components_metadata";

import { EditorContext, SchematicKind } from "@/api/sdf/dal/schematic";
import { SceneManager } from "./Viewer/scene_manager";
import { InteractionManager } from "./Viewer/interaction_manager";
import { Renderer } from "./Viewer/renderer";
import { SchematicDataManager } from "./data_manager";
import { Schematic, SchematicNode, variantById } from "@/api/sdf/dal/schematic";
import { nodeSelection$, SelectedNode } from "@/observable/selection";

import * as VE from "./viewer_event";

const props = defineProps<{
  schematicViewerId: string;
  viewerState: SM.ViewerStateMachine;
  viewerEvent$: Rx.ReplaySubject<VE.ViewerEvent | null>;
  schematicData: Schematic;
  editorContext: EditorContext | null;
  schematicKind: SchematicKind;
  deploymentNodeSelected: number | null;
  lightMode?: boolean;
}>();

const machine = useMachine(props.viewerState.machine);
const { state, send } = machine;
const service = machine.service as Interpreter<unknown>;

const dataManager = new SchematicDataManager();
dataManager.editorContext$.next(props.editorContext);
dataManager.schematicKind$.next(props.schematicKind);
dataManager.selectedDeploymentNodeId$.next(props.deploymentNodeSelected);

const component = {
  id: _.uniqueId(),
  isActive: false,
};
const canvas = {
  id: props.schematicViewerId + ":" + component.id,
  element: ref<HTMLCanvasElement | undefined>(undefined),
  isPanning: false,
  mousePosition: undefined,
};
const container = {
  id: props.schematicViewerId + "-" + "container",
  element: ref<HTMLElement | undefined>(undefined),
};
const debug = false;

// Temporaries that will be replaced when the rest of the data is available (onMounted)
// As the renderer depends on ref elements (canvas, etc) and everything depends on renderer
let maybeRenderer: Renderer | undefined = undefined;
let maybeSceneManager: SceneManager | undefined = undefined;
let maybeInteractionManager: InteractionManager | undefined = undefined;

// Watch for element resize. Not completely smooth, will need to revisit.
const resizeObserver = new ResizeObserver((entries) => {
  resizeCanvas(entries[0].contentRect.width, entries[0].contentRect.height);
});

const { deploymentNodeSelected, editorContext, schematicKind, schematicData } =
  toRefs(props);

watch(editorContext, (ctx) => {
  dataManager.editorContext$.next(ctx);
});
watch(schematicData, async (schematic) => {
  dataManager.schematicData$.next(schematic);
});

watch(deploymentNodeSelected, async (ctx) => {
  dataManager.selectedDeploymentNodeId$.next(ctx);
  if (maybeSceneManager && maybeRenderer) {
    await loadSchematicData(
      props.schematicData,
      maybeSceneManager,
      maybeRenderer,
    );
  }
});

watch(schematicKind, async (ctx) => {
  dataManager.schematicKind$.next(ctx);
  if (maybeSceneManager && maybeRenderer) {
    await loadSchematicData(
      props.schematicData,
      maybeSceneManager,
      maybeRenderer,
    );
  }

  // Fixes some inconsistencies where if you change panels while dragging the dragging would stick forever
  if (maybeInteractionManager) {
    SM.deactivateDragging(maybeInteractionManager.stateService);
  }
});

onMounted(async () => {
  document.addEventListener("keydown", handleKeyDown);
  document.addEventListener("keyup", handleKeyUp);

  if (!canvas.element.value) throw new Error("canvas is missing");
  if (!container.element.value) throw new Error("container is missing");
  resizeObserver.observe(container.element.value as Element);

  // Check for light mode in advance.
  let lightMode = false;
  if (props.lightMode) {
    lightMode = true;
  }

  let backgroundColor = 0x282828;
  if (lightMode) {
    // RGB(244,244,244)
    backgroundColor = 0xf4f4f4;
  }

  const renderer = new Renderer({
    view: canvas.element.value,
    resolution: window.devicePixelRatio * 3 || 1,
    width: container.element.value.offsetWidth,
    height: container.element.value.offsetHeight,
    autoDensity: true,
    antialias: true,
    backgroundColor: backgroundColor,
  });
  maybeRenderer = renderer;

  // Handles elements that appear on the screen
  const sceneManager = new SceneManager(renderer, lightMode);
  maybeSceneManager = sceneManager;
  renderer.stage.addChild(sceneManager.scene);

  // Re-renders every time the dataset changes
  dataManager.schematicData$.pipe(untilUnmounted).subscribe(async (data) => {
    if (data) await loadSchematicData(data, sceneManager, renderer);
  });

  // Local selection status needs to update when global selection status does
  nodeSelection$.pipe(untilUnmounted).subscribe((selections) => {
    syncSelection(selections, renderer, sceneManager);
  });

  // Updates local status of resource syncing and qualification checks
  // When global metadata updates
  componentsMetadata$.pipe(untilUnmounted).subscribe((metadatas) => {
    if (metadatas) updateMetadata(metadatas, sceneManager, renderer);
  });

  // Handles operations done to the schematic (create/move node, create connection, etc)
  const interactionManager = new InteractionManager(
    sceneManager,
    dataManager,
    service,
    renderer,
  );
  maybeInteractionManager = interactionManager;
  sceneManager.subscribeToInteractionEvents(interactionManager);

  // Act on events like NODE_ADD
  props.viewerEvent$.pipe(untilUnmounted).subscribe((e) => {
    if (e?.kind === VE.ViewerEventKind.NODE_ADD) {
      nodeAdd(e.data.node, e.data.schemaId, interactionManager);
    }
  });

  // Let's prepare the data to be rendered and render it
  await loadSchematicData(props.schematicData, sceneManager, renderer);
  renderer.renderStage();

  // Note: Horrible hack. I have no idea why, but without this the first load doesn't show the qualification/resource sync icons
  setTimeout(async () => {
    const metadatas = await Rx.firstValueFrom(componentsMetadata$);
    if (metadatas) updateMetadata(metadatas, sceneManager, renderer);
  }, 100);
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeyDown);
  document.removeEventListener("keyup", handleKeyUp);

  if (container.element.value) {
    resizeObserver.unobserve(container.element.value);
  }
});

async function loadSchematicData(
  schematic: Schematic,
  sceneManager: SceneManager,
  renderer: Renderer,
): Promise<void> {
  await sceneManager.loadSceneData(
    schematic,
    props.schematicKind,
    deploymentNodeSelected.value,
  );

  syncSelection(
    await Rx.firstValueFrom(nodeSelection$),
    renderer,
    sceneManager,
  );

  const metadatas = await Rx.firstValueFrom(componentsMetadata$);
  if (metadatas) updateMetadata(metadatas, sceneManager, renderer);
}

function updateMetadata(
  metadatas: ComponentMetadata[],
  sceneManager: SceneManager,
  renderer: Renderer,
) {
  for (const metadata of metadatas) {
    for (const n of sceneManager.group.nodes.children) {
      const node = n as Node;
      if (metadata.componentId === node.nodeKind?.componentId) {
        node.setQualificationStatus(metadata.qualified);
        node.setResourceStatus(metadata.resourceHealth);
        break;
      }
    }
  }
  renderer.renderStage();
}
async function nodeAdd(
  node: SchematicNode,
  schemaId: number,
  interactionManager: InteractionManager,
): Promise<void> {
  activateComponent();

  if (component.isActive && node.positions.length > 0) {
    const schemaVariant = await variantById(node.schemaVariantId);

    // Fake nodes will always only have one position as they don't exist in the db yet
    const nodeObj = new Node(
      node,
      schemaVariant,
      {
        x: node.positions[0].x,
        y: node.positions[0].y,
      },
      props.schematicKind,
    );
    send(SM.ViewerEventKind.ACTIVATE_NODEADD);
    interactionManager.nodeAddManager.addNode(nodeObj, schemaId);
  }
}

function syncSelection(
  selections: SelectedNode[],
  renderer: Renderer,
  sceneManager: SceneManager,
) {
  const nodes = sceneManager.group.nodes.children as Node[];
  const selected = selections.find(
    (s) => s.parentDeploymentNodeId === deploymentNodeSelected.value,
  );

  for (const node of nodes ?? []) {
    if (selected?.nodeIds?.includes(node.id)) {
      node.select();
    } else {
      node.deselect();
    }
  }

  renderer.renderStage();
}

function resizeCanvas(width: number, height: number): void {
  if (maybeRenderer && container.element.value) {
    maybeRenderer.resize(width, height);
    maybeRenderer.renderStage();
  }
}

const spacebarKey = {
  long: "Spacebar",
  short: " ",
};

function handleKeyDown(e: KeyboardEvent): void {
  if (e.key === spacebarKey.long || e.key === spacebarKey.short) {
    e.preventDefault();
    if (component.isActive) {
      send(SM.ViewerEventKind.ACTIVATE_PANNING);
    }
  }
}

function handleKeyUp(e: KeyboardEvent): void {
  if (e.key === spacebarKey.long || e.key === spacebarKey.short) {
    e.preventDefault();
    send(SM.ViewerEventKind.DEACTIVATE_PANNING);
  }
}

function handleMouseWheel(e: WheelEvent) {
  e.preventDefault();
  // implement zoom on alt/option key
  send(SM.ViewerEventKind.ACTIVATE_ZOOMING);
  send(SM.ViewerEventKind.INITIATE_ZOOMING);
  send(SM.ViewerEventKind.ZOOMING);
  maybeInteractionManager?.zoomingManager?.zoom(e);
  maybeRenderer?.renderStage();
  send(SM.ViewerEventKind.DEACTIVATE_ZOOMING);
}

function activateComponent(): void {
  component.isActive = true;
}

function deactivateComponent(): void {
  component.isActive = false;
}

function mouseEnter(): void {
  activateComponent();
}
function mouseLeave(): void {
  deactivateComponent();
}
</script>
