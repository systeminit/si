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
      <ol
        id="construction"
        class="flex flex-row w-full mb-md border border-neutral-600 p-sm"
      >
        <template
          v-for="(cmd, index) in commandBuffer"
          :key="`${cmd}-${index}`"
        >
          <li
            class="cmd buffer mr-xs text-md rounded-md bg-action-900 px-xs py-[.33rem]"
            @click="() => remove(cmd, index)"
          >
            {{ cmd.name }}
          </li>

          <li
            v-for="(choice, indexChoice) in cmd.choices"
            :key="`${choice.value}-${indexChoice}`"
            :class="
              clsx(
                'buffer mr-xs text-md rounded-md bg-action-500 px-xs py-[.33rem]',
                indexChoice + 1 === cmd.choices.length ? 'last' : '',
              )
            "
            @click="
              () =>
                indexChoice + 1 === cmd.choices.length
                  ? removeChoice(cmd, index, choice, indexChoice)
                  : null
            "
          >
            {{ choice.label }}
          </li>
        </template>
        <li v-if="!maybeDone" class="text-lg grow relative">
          <input
            id="commandInput"
            ref="commandInputRef"
            v-model="commandInput"
            autocomplete="off"
            spellcheck="false"
            role="presentation"
            placeholder="Type your command..."
            class="w-full"
            @keydown.enter="input"
            @keydown="filter"
          />
          <ul
            v-if="dropDownOptions.length > 0"
            id="dropdown"
            ref="dropdown"
            :style="dropdownStyle"
            :class="
              clsx(
                'absolute top-8 left-0 text-sm',
                'rounded-sm min-h-[15vh] max-h-[30vh] min-w-[10vh] max-w-[15vh] overflow-y-auto',
                themeClasses(
                  'bg-white text-black',
                  'bg-neutral-900 text-white',
                ),
              )
            "
          >
            <li
              v-for="option in filteredDropDownOptions"
              :key="option.value"
              class="p-xs"
              :class="
                clsx(
                  selectedOption?.value === option.value
                    ? themeClasses('text-action-500', 'text-action-300')
                    : '',
                  selectedOption?.value === option.value
                    ? themeClasses('bg-neutral-300', 'bg-neutral-600')
                    : '',
                )
              "
              @click="chooseArg(option)"
            >
              {{ option.label }}
            </li>
            <li v-if="filteredDropDownOptions.length === 0" class="ps-x italic">
              No options
            </li>
          </ul>
        </li>
        <li v-else>
          <VButton
            ref="go"
            tone="action"
            variant="solid"
            size="sm"
            @keydown.enter="runGo"
            @click="runGo"
          >
            go
          </VButton>
        </li>
      </ol>
      <h5>Command list:</h5>
      <ul>
        <li>[P]an to &lt;component&gt;</li>
        <li>
          [C]onnect &lt;component&gt; &lt;socket&gt; to &lt;component&gt;
          &lt;socket&gt;
        </li>
        <li>[Q]ueue a &lt;component&gt; &lt;action&gt;</li>
      </ul>
    </section>
  </Modal>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { ref, computed, onBeforeUnmount, onMounted, watch, toRaw } from "vue";
import { Modal, VButton, themeClasses } from "@si/vue-lib/design-system";
import { connectionAnnotationFitsReference } from "@si/ts-lib/src/connection-annotations";
import { useComponentsStore } from "@/store/components.store";
import { useActionsStore } from "@/store/actions.store";
import {
  useFuncStore,
  actionBindingsForVariant,
  BindingWithDisplayName,
} from "@/store/func/funcs.store";
import { Action, FuncId, FuncSummary } from "@/api/sdf/dal/func";
import { ActionPrototypeId } from "@/api/sdf/dal/action";

const actionsStore = useActionsStore();
const componentStore = useComponentsStore();
const funcStore = useFuncStore();

const go = ref<InstanceType<typeof VButton>>();
const commandModal = ref<InstanceType<typeof Modal>>();
const commandInputRef = ref<InstanceType<typeof HTMLInputElement>>();
const commandInput = ref<string>("");
const dropdown = ref<InstanceType<typeof HTMLUListElement>>();
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const dropdownStyle = ref<any>({});

watch(go, (newGo, oldGo) => {
  // whenever newGo enters the stage, focus it so user can hit enter
  if (!oldGo && newGo) {
    newGo.focus();
  }
});

type VoidFn = () => void;
type FactoryFn = () => Command;
type CommandArg =
  | "component"
  | "outputSocket"
  | "inputSocket"
  | "schema"
  | "action";
interface Command {
  readonly name: string;
  readonly shortcut: string;
  readonly expects: CommandArg[];
  choices: Option[];
  execute: VoidFn;
  factory: FactoryFn;
}

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
    componentStore.setSelectedComponentId(componentId);
  }
  // I can't make this static because the instance won't have a reference to it
  // eslint-disable-next-line class-methods-use-this
  factory() {
    return new Pan();
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
const Commands: Command[] = [new Pan(), new Queue(), new Connect()];

const commandBuffer = ref<Command[]>([]);
const lastCmd = computed<Command | undefined>(
  () => commandBuffer.value[commandBuffer.value.length - 1],
);

type Option = { label: string; value: string };

const dropDownOptions = ref<Option[]>([]);

export interface ActionBindingWithPrototype extends BindingWithDisplayName {
  actionPrototypeId: NonNullable<ActionPrototypeId>;
}

const setDropDown = () => {
  const source = lastCmd.value?.expects[lastCmd.value?.choices.length];
  if (!source) dropDownOptions.value = [];
  switch (source) {
    case "component":
      dropDownOptions.value = Object.values(
        componentStore.rawComponentsById,
      ).map((c) => ({ label: c.displayName, value: c.id }));
      break;
    case "action":
      dropDownOptions.value = [];
      if (lastCmd.value) {
        const idx = lastCmd.value.choices.length - 1;
        const prevSource = lastCmd.value.expects[idx];
        const prevChoice = lastCmd.value.choices[idx];
        if (prevSource === "component" && prevChoice) {
          const component = componentStore.rawComponentsById[prevChoice.value];
          const variant =
            componentStore.schemaVariantsById[component?.id || ""];
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
      if (!lastCmd.value) {
        dropDownOptions.value = [];
      } else {
        const idx = lastCmd.value.choices.length - 1;
        const prevSource = lastCmd.value.expects[idx];
        const prevChoice = lastCmd.value.choices[idx];
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
      if (!lastCmd.value) {
        dropDownOptions.value = [];
      } else {
        const cIdx = lastCmd.value.choices.length - 1;
        const maybeComponent = lastCmd.value.expects[cIdx];
        const prevComponent = lastCmd.value.choices[cIdx];
        const sIdx = lastCmd.value.choices.length - 2;
        const maybeSocket = lastCmd.value.expects[sIdx];
        const prevOutputSocket = lastCmd.value.choices[sIdx];
        const oIdx = lastCmd.value.choices.length - 3;
        const maybeOriginComponent = lastCmd.value.expects[oIdx];
        const prevOriginComponent = lastCmd.value.choices[oIdx];
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
      dropDownOptions.value = Object.values(
        componentStore.schemaVariantsById,
      ).map((s) => ({ label: s.schemaName, value: s.schemaVariantId }));
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

const dropDownFilter = ref<string | null>();
const selectedOption = ref<Option | null>();

const filter = (event: KeyboardEvent) => {
  let idx = filteredDropDownOptions.value.findIndex(
    (o) => o.value === selectedOption.value?.value,
  );
  if (event.code === "ArrowDown") {
    if (idx === null) idx = 0;
    else {
      idx++;
      if (idx >= filteredDropDownOptions.value.length) idx = 0;
    }
    selectedOption.value = filteredDropDownOptions.value[idx];
  } else if (event.code === "ArrowUp") {
    if (idx === null) idx = filteredDropDownOptions.value.length - 1;
    else {
      idx--;
      if (idx <= 0) idx = filteredDropDownOptions.value.length - 1;
    }
    selectedOption.value = filteredDropDownOptions.value[idx];
  } else {
    // assuming letters, ignore enter, etc
    dropDownFilter.value = commandInput.value;
  }
};

const filteredDropDownOptions = computed(() => {
  const filter = dropDownFilter.value?.trim().toLocaleLowerCase();
  if (!filter || filter.length === 0) return dropDownOptions.value;
  return dropDownOptions.value.filter((o) => {
    return o.label.toLocaleLowerCase().includes(filter);
  });
});

const chooseArg = (choice: Option) => {
  lastCmd.value?.choices.push(choice);
  input();
};

const remove = (cmd: Command, index: number) => {
  commandBuffer.value.splice(index, 1);
  dropDownOptions.value = [];
};

const removeChoice = (
  cmd: Command,
  index: number,
  choice: Option,
  indexChoice: number,
) => {
  cmd.choices.splice(indexChoice, 1);
  setDropDown();
};

const maybeDone = computed(
  () =>
    lastCmd.value &&
    lastCmd.value.expects.length === lastCmd.value.choices.length,
);

watch(maybeDone, (newDone) => {
  if (newDone) dropDownOptions.value = [];
});

const runGo = async () => {
  if (!maybeDone.value) return;
  else {
    await lastCmd.value?.execute();
    commandModal.value?.close();
  }
};

const input = (event?: Event) => {
  event?.preventDefault();
  const str = commandInput.value;

  // am i setting a choice for an arg?
  if (lastCmd.value) {
    if (selectedOption.value) {
      lastCmd.value.choices.push(structuredClone(toRaw(selectedOption.value)));
      selectedOption.value = null;

      // are there more left? set drop down for the next choice
      if (!maybeDone.value) {
        setDropDown();
      }
    } else if (lastCmd.value.expects.length !== lastCmd.value.choices.length) {
      let choice = dropDownOptions.value.find((o) => o.label === str);
      if (!choice) choice = dropDownOptions.value.find((o) => o.value === str);
      if (choice) lastCmd.value.choices.push(structuredClone(choice));

      // are there more left? set drop down for the next choice
      if (!maybeDone.value) {
        setDropDown();
      }
    }
  }

  // am i starting a new command?
  if (commandBuffer.value.length === 0) {
    if (!str) return;
    for (const cmd of Commands) {
      if (str === cmd.name || str === cmd.shortcut) {
        commandBuffer.value.push(cmd.factory());
        setDropDown();
      }
    }
  }

  // reset input
  if (commandInput.value) commandInput.value = "";

  // TODO are we done? go!
  if (maybeDone.value) {
    go.value?.focus();
  } else if (commandInput.value) {
    // keep focus
    commandInputRef.value?.focus();
  }
};

const onKeyDown = async (e: KeyboardEvent) => {
  if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
    if (!commandModal.value?.isOpen) commandModal.value?.open();
    else commandModal.value?.close();
  }
};

const close = () => {
  commandBuffer.value = [];
  dropDownOptions.value = [];
  dropDownFilter.value = null;
  selectedOption.value = null;
};

onMounted(() => {
  window.addEventListener("keydown", onKeyDown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeyDown);
});
</script>

<style lang="less">
#commandInput {
  outline: 0;
  background-color: transparent;
}

#construction .buffer {
  cursor: pointer;
}

#construction .cmd.buffer,
#construction .last.buffer {
  cursor: not-allowed;
}

#dropdown {
  border: 1px solid #ccc;
  border-top: 0;
}

#dropdown > * {
  cursor: pointer;
  border-top: 1px solid #ccc;
}
</style>
