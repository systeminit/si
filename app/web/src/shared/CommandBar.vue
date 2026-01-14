<template>
  <ol id="construction" class="flex flex-row w-full mb-md border border-neutral-600 p-sm">
    <template v-for="(cmd, index) in commandBuffer" :key="`${cmd}-${index}`">
      <li class="cmd buffer mr-xs text-md rounded-md bg-action-900 px-xs py-[.33rem]" @click="() => remove(cmd, index)">
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
        @click="() => (indexChoice + 1 === cmd.choices.length ? removeChoice(cmd, index, choice, indexChoice) : null)"
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
            themeClasses('bg-white text-black', 'bg-neutral-900 text-white'),
          )
        "
      >
        <li
          v-for="option in filteredDropDownOptions"
          :key="option.value"
          class="p-xs"
          :class="
            clsx(
              selectedOption?.value === option.value ? themeClasses('text-action-500', 'text-action-300') : '',
              selectedOption?.value === option.value ? themeClasses('bg-neutral-300', 'bg-neutral-600') : '',
            )
          "
          @click="chooseArg(option)"
        >
          {{ option.label }}
        </li>
        <li v-if="filteredDropDownOptions.length === 0" class="ps-x italic">No options</li>
      </ul>
    </li>
    <li v-else>
      <VButton ref="go" tone="action" variant="solid" size="sm" @keydown.enter="runGo" @click="runGo"> go </VButton>
    </li>
  </ol>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { ref, watch, computed, toRaw, ComputedRef, Ref } from "vue";
import { VButton, themeClasses } from "@si/vue-lib/design-system";
import { Command, Option } from "@/shared/command";

const go = ref<InstanceType<typeof VButton>>();
const commandInputRef = ref<InstanceType<typeof HTMLInputElement>>();
const commandInput = ref<string>("");
const dropdown = ref<InstanceType<typeof HTMLUListElement>>();
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const dropdownStyle = ref<any>({});

interface Props {
  commands: Command[];
  setDropDown: (command: ComputedRef<Command | undefined>, dropdownOptions: Ref<Option[]>) => void;
}
const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "done"): void;
}>();

const reset = () => {
  commandBuffer.value = [];
  dropDownOptions.value = [];
  dropDownFilter.value = null;
  selectedOption.value = null;
};

defineExpose({
  reset,
});

watch(go, (newGo, oldGo) => {
  // whenever newGo enters the stage, focus it so user can hit enter
  if (!oldGo && newGo) {
    newGo.focus();
  }
});

const commandBuffer = ref<Command[]>([]);
const lastCmd = computed<Command | undefined>(() => commandBuffer.value[commandBuffer.value.length - 1]);

const dropDownOptions = ref<Option[]>([]);

const dropDownFilter = ref<string | null>();
const selectedOption = ref<Option | null>();

const filter = (event: KeyboardEvent) => {
  let idx = filteredDropDownOptions.value.findIndex((o) => o.value === selectedOption.value?.value);
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

const removeChoice = (cmd: Command, index: number, choice: Option, indexChoice: number) => {
  cmd.choices.splice(indexChoice, 1);
  props.setDropDown(lastCmd, dropDownOptions);
};

const maybeDone = computed(() => lastCmd.value && lastCmd.value.expects.length === lastCmd.value.choices.length);

watch(maybeDone, (newDone) => {
  if (newDone) dropDownOptions.value = [];
});

const runGo = () => {
  if (!maybeDone.value) return;
  else {
    lastCmd.value?.execute();
    reset();
    emit("done");
  }
};

const input = (event?: KeyboardEvent) => {
  event?.preventDefault();
  const str = commandInput.value;
  const numChoices = lastCmd.value?.choices.length ?? 0;

  // am i setting a choice for an arg?
  if (lastCmd.value) {
    if (selectedOption.value) {
      lastCmd.value.choices.push(structuredClone(toRaw(selectedOption.value)));
      selectedOption.value = null;

      // are there more left? set drop down for the next choice
      if (!maybeDone.value) {
        props.setDropDown(lastCmd, dropDownOptions);
      }
    } else if (lastCmd.value.expects.at(numChoices) === "stringInput") {
      // typing free text, not a dropdown choice
      if (event?.key === "Enter") {
        if (str) lastCmd.value.choices.push({ label: str, value: str });
      }
    } else if (lastCmd.value.expects.length !== numChoices) {
      let choice = dropDownOptions.value.find((o) => o.label === str);
      if (!choice) choice = dropDownOptions.value.find((o) => o.value === str);
      if (choice) lastCmd.value.choices.push(structuredClone(choice));

      // are there more left? set drop down for the next choice
      if (!maybeDone.value) {
        props.setDropDown(lastCmd, dropDownOptions);
      }
    }
  }

  // am i starting a new command?
  if (commandBuffer.value.length === 0) {
    if (!str) return;
    for (const cmd of props.commands) {
      if (str === cmd.name || str === cmd.shortcut) {
        commandBuffer.value.push(cmd.factory());
        props.setDropDown(lastCmd, dropDownOptions);
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
