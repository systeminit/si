<template>
  <Modal ref="commandModal" noWrapper size="4wxl" @close="close">
    <section
      :class="
        clsx(
          'rounded-md p-md',
          themeClasses('text-black bg-white', 'bg-neutral-900 text-white'),
        )
      "
    >
      <CommandBar
        ref="commandBar"
        :commands="Commands"
        :setDropDown="setDropDown"
        @done="() => commandModal?.close()"
      />
      <h5>Command list:</h5>
      <ul>
        <li>[P]an to &lt;component&gt;</li>
        <li>
          [C]onnect &lt;component&gt; &lt;socket&gt; to &lt;component&gt;
          &lt;socket&gt;
        </li>
        <li>[Q]ueue a &lt;component&gt; &lt;action&gt;</li>
        <li>[G]oto a &lt;view&gt;</li>
        <li>[N]ew &lt;view&gt;</li>
      </ul>
    </section>
  </Modal>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { ref, onBeforeUnmount, onMounted, ComputedRef, Ref } from "vue";
import { Modal, themeClasses } from "@si/vue-lib/design-system";
import { connectionAnnotationFitsReference } from "@si/ts-lib/src/connection-annotations";
import { useComponentsStore } from "@/store/components.store";
import { useActionsStore } from "@/store/actions.store";
import { useAssetStore } from "@/store/asset.store";
import { Command, CommandArg, Option } from "@/shared/command";
import CommandBar from "@/shared/CommandBar.vue";
import {
  useFuncStore,
  actionBindingsForVariant,
  BindingWithDisplayName,
} from "@/store/func/funcs.store";
import { Action, FuncId, FuncSummary } from "@/api/sdf/dal/func";
import { ActionPrototypeId } from "@/api/sdf/dal/action";
import { useViewsStore } from "@/store/views.store";

const actionsStore = useActionsStore();
const componentStore = useComponentsStore();
const viewStore = useViewsStore();
const funcStore = useFuncStore();
const assetStore = useAssetStore();

const commandModal = ref<InstanceType<typeof Modal>>();
const commandBar = ref<InstanceType<typeof CommandBar>>();

// NOTE: we need this interface to prevent duplicating properties
// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface Pan extends Command {}
class Pan implements Command {
  name = "Pan";
  shortcut = "P";
  expects: CommandArg[] = ["component"];
  constructor() {
    this.choices = [];
  }
  execute() {
    const componentId = this.choices[0]?.value;
    if (!componentId) throw new Error("ComponentId Expected");
    componentStore.panTargetComponentId = componentId;
    viewStore.setSelectedComponentId(componentId);
  }
  // I can't make this static because the instance won't have a reference to it
  // eslint-disable-next-line class-methods-use-this
  factory() {
    return new Pan();
  }
}

// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface GotoView extends Command {}
class GotoView implements Command {
  name = "Goto View";
  shortcut = "G";
  expects: CommandArg[] = ["view"];
  constructor() {
    this.choices = [];
  }
  execute() {
    const viewId = this.choices[0]?.value;
    if (!viewId) throw new Error("ViewId Expected");
    viewStore.selectView(viewId);
  }
  // I can't make this static because the instance won't have a reference to it
  // eslint-disable-next-line class-methods-use-this
  factory() {
    return new GotoView();
  }
}

// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface NewView extends Command {}
class NewView implements Command {
  name = "New View";
  shortcut = "N";
  expects: CommandArg[] = ["stringInput"];
  constructor() {
    this.choices = [];
  }
  execute() {
    const name = this.choices[0]?.value;
    if (!name) throw new Error("Expected name");
    viewStore.CREATE_VIEW(name).then((resp) => {
      if (resp.result.success) viewStore.selectView(resp.result.data.id);
    });
  }
  // I can't make this static because the instance won't have a reference to it
  // eslint-disable-next-line class-methods-use-this
  factory() {
    return new NewView();
  }
}

// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface Queue extends Command {}
class Queue implements Command {
  name = "Queue";
  shortcut = "Q";
  expects: CommandArg[] = ["component", "action"];
  constructor() {
    this.choices = [];
  }
  execute() {
    const componentId = this.choices[0]?.value;
    const actionPrototypeId = this.choices[1]?.value;
    if (!componentId || !actionPrototypeId) return;

    const action = actionsStore.listActionsByComponentId
      .get(componentId)
      .find((a) => a.prototypeId === actionPrototypeId);

    if (action?.id) {
      actionsStore.CANCEL([action.id]);
    } else if (actionPrototypeId) {
      actionsStore.ADD_ACTION(componentId, actionPrototypeId);
    }
  }
  // I can't make this static because the instance won't have a reference to it
  // eslint-disable-next-line class-methods-use-this
  factory() {
    return new Queue();
  }
}

// eslint-disable-next-line @typescript-eslint/no-empty-interface
interface Connect extends Command {}
class Connect implements Command {
  name = "Connect";
  shortcut = "C";
  expects: CommandArg[] = [
    "component",
    "outputSocket",
    "component",
    "inputSocket",
  ];
  constructor() {
    this.choices = [];
  }
  execute() {
    const fromComponentId = this.choices[0]?.value;
    const fromSocketId = this.choices[1]?.value;
    const toComponentId = this.choices[2]?.value;
    const toSocketId = this.choices[3]?.value;
    if (!fromComponentId || !fromSocketId || !toComponentId || !toSocketId)
      throw new Error("Values Missing");
    componentStore.CREATE_COMPONENT_CONNECTION(
      { componentId: fromComponentId, socketId: fromSocketId },
      { componentId: toComponentId, socketId: toSocketId },
    );
  }
  // I can't make this static because the instance won't have a reference to it
  // eslint-disable-next-line class-methods-use-this
  factory() {
    return new Connect();
  }
}
const Commands: Command[] = [
  new Pan(),
  new Queue(),
  new Connect(),
  new GotoView(),
  new NewView(),
];

export interface ActionBindingWithPrototype extends BindingWithDisplayName {
  actionPrototypeId: NonNullable<ActionPrototypeId>;
}

const setDropDown = (
  cmd: ComputedRef<Command | undefined>,
  dropDownOptions: Ref<Option[]>,
) => {
  const source = cmd.value?.expects[cmd.value?.choices.length];
  if (!source) dropDownOptions.value = [];
  switch (source) {
    case "stringInput":
      dropDownOptions.value = [];
      break;
    case "view":
      dropDownOptions.value = Object.values(viewStore.viewList).map((c) => ({
        label: c.name,
        value: c.id,
      }));
      break;
    case "component":
      dropDownOptions.value = Object.values(
        componentStore.rawComponentsById,
      ).map((c) => ({ label: c.displayName, value: c.id }));
      break;
    case "action":
      dropDownOptions.value = [];
      if (cmd.value) {
        const idx = cmd.value.choices.length - 1;
        const prevSource = cmd.value.expects[idx];
        const prevChoice = cmd.value.choices[idx];
        if (prevSource === "component" && prevChoice) {
          const component = componentStore.rawComponentsById[prevChoice.value];
          const variant =
            assetStore.variantFromListById[component?.schemaVariantId || ""];
          if (variant) {
            const summaries: Record<FuncId, FuncSummary> = {};
            const actionBindings: Record<FuncId, Action[]> = {};
            variant?.funcIds.forEach((funcId) => {
              const summary = funcStore.funcsById[funcId];
              if (summary) summaries[funcId] = summary;
              const actions = funcStore.actionBindings[funcId];
              if (actions) actionBindings[funcId] = actions;
            });
            dropDownOptions.value = actionBindingsForVariant(
              variant,
              summaries,
              actionBindings,
            )
              .filter(
                (binding): binding is ActionBindingWithPrototype =>
                  !!binding.actionPrototypeId,
              )
              .map((binding) => {
                const option: Option = {
                  value: binding.actionPrototypeId,
                  label: binding.displayName || binding.name,
                };
                return option;
              });
          }
        }
      }
      break;
    case "outputSocket":
      if (!cmd.value) {
        dropDownOptions.value = [];
      } else {
        const idx = cmd.value.choices.length - 1;
        const prevSource = cmd.value.expects[idx];
        const prevChoice = cmd.value.choices[idx];
        if (prevSource === "component" && prevChoice) {
          dropDownOptions.value =
            componentStore.rawComponentsById[prevChoice.value]?.sockets
              .filter((s) => s.direction === "output")
              .map((s) => ({
                label: s.label,
                value: s.id,
              })) || [];
        } else
          throw Error(
            `Unexpected source ${prevSource} and choice ${prevChoice}`,
          );
      }
      break;
    case "inputSocket":
      if (!cmd.value) {
        dropDownOptions.value = [];
      } else {
        const cIdx = cmd.value.choices.length - 1;
        const maybeComponent = cmd.value.expects[cIdx];
        const prevComponent = cmd.value.choices[cIdx];
        const sIdx = cmd.value.choices.length - 2;
        const maybeSocket = cmd.value.expects[sIdx];
        const prevOutputSocket = cmd.value.choices[sIdx];
        const oIdx = cmd.value.choices.length - 3;
        const maybeOriginComponent = cmd.value.expects[oIdx];
        const prevOriginComponent = cmd.value.choices[oIdx];
        const outputSocket = componentStore.rawComponentsById[
          prevOriginComponent?.value || ""
        ]?.sockets.find((s) => s.id === prevOutputSocket?.value || "");
        if (
          maybeComponent === "component" &&
          prevComponent &&
          maybeSocket === "outputSocket" &&
          prevOutputSocket &&
          maybeOriginComponent === "component" &&
          prevOriginComponent &&
          outputSocket
        ) {
          dropDownOptions.value =
            componentStore.rawComponentsById[prevComponent.value]?.sockets
              .filter((s) => s.direction === "input")
              .filter((s) => {
                for (const outputCA of outputSocket.connectionAnnotations)
                  for (const inputCA of s.connectionAnnotations) {
                    if (connectionAnnotationFitsReference(outputCA, inputCA)) {
                      return true;
                    }
                  }
                return false;
              })
              .map((s) => ({
                label: s.label,
                value: s.id,
              })) || [];
        } else {
          dropDownOptions.value = [];
          throw Error(
            `Unexpected sources ${maybeComponent}, ${maybeSocket} and choices ${prevComponent}, ${prevOutputSocket}`,
          );
        }
      }
      break;
    case "schema":
      dropDownOptions.value = assetStore.variantList.map((s) => ({
        label: s.schemaName,
        value: s.schemaVariantId,
      }));
      break;
    default:
      dropDownOptions.value = [];
      break;
  }
  dropDownOptions.value.sort((a, b) => {
    if (a.label < b.label) return -1;
    if (a.label > b.label) return 1;
    return 0;
  });
};

const onKeyDown = async (e: KeyboardEvent) => {
  if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
    if (!commandModal.value?.isOpen) commandModal.value?.open();
    else commandModal.value?.close();
  }
};

const close = () => {
  commandBar.value?.reset();
};

onMounted(() => {
  window.addEventListener("keydown", onKeyDown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeyDown);
});
</script>
