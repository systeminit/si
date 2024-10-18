<template>
  <th
    ref="thRef"
    v-tooltip="headerTooltip"
    :colSpan="header.colSpan"
    :class="
      clsx(
        'h-8 sticky top-0',
        header.id !== 'ip' && 'cursor-pointer hover:underline',
        themeClasses('bg-shade-0', 'bg-shade-100'),
      )
    "
    @mousedown="startActive"
    @mouseup="endActive"
    @click.stop="onClick"
  >
    <div class="w-full p-xs truncate">
      <FlexRender
        v-if="!header.isPlaceholder"
        :render="label"
        :props="header.getContext()"
      />
      <IconButton
        v-if="icon !== 'none'"
        ref="iconButtonRef"
        class="absolute right-xs top-2xs"
        :icon="icon"
        iconTone="neutral"
        @click.stop="onClick"
      />
      <DropdownMenu
        v-if="header.id !== 'timestamp'"
        ref="dropdownMenuRef"
        :items="dropdownMenuItems"
        :anchorTo="{ $el: thRef }"
        alignCenter
      />
    </div>
  </th>
</template>

<script lang="ts" setup>
import {
  DropdownMenu,
  DropdownMenuItemObjectDef,
  IconButton,
  themeClasses,
} from "@si/vue-lib/design-system";
import { FlexRender, Header } from "@tanstack/vue-table";
import clsx from "clsx";
import { computed, PropType, ref } from "vue";
import { AuditLogKind, AuditLogService, LogFilters } from "@/store/logs.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { AdminUser } from "@/store/admin.store";

const changeSetsStore = useChangeSetsStore();

const thRef = ref();
const iconButtonRef = ref<InstanceType<typeof IconButton>>();
const dropdownMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const props = defineProps({
  header: {
    type: Object as PropType<
      Header<
        {
          actorId: string;
          actorName: string;
          actorEmail?: string | undefined;
          service: string;
          kind: string;
          timestamp: string;
          ip: string;
          changeSetId: string;
          changeSetName: string;
        },
        unknown
      >
    >,
    required: true,
  },
  filters: {
    type: Object as PropType<LogFilters>,
    required: true,
  },
  users: {
    type: Array as PropType<AdminUser[]>,
    default: [] as AdminUser[],
  },
});

const label = computed(() => props.header.column.columnDef.header as string);

const icon = computed(() => {
  if (props.header.id === "timestamp") {
    if (props.filters.sortTimestampAscending) return "chevron--up";
    else return "chevron--down";
  } else if (selectedFilters.value.length > 0) {
    return "filter";
  } else {
    return "none";
  }
});

const filterOptions = computed(() => {
  if (props.header.id === "kind") {
    return Object.values(AuditLogKind).map((k) => {
      return { label: k, value: k };
    });
  } else if (props.header.id === "service") {
    return Object.values(AuditLogService).map((k) => {
      return { label: k, value: k };
    });
  } else if (props.header.id === "changeSetName") {
    return changeSetsStore.allChangeSets.map((changeSet) => {
      return { label: changeSet.name, value: changeSet.id };
    });
  } else if (props.header.id === "actorName") {
    const actors = props.users.map((user) => {
      return { label: user.name, value: user.id };
    });
    actors.unshift({ label: "System", value: "System" });

    return actors;
  }
  return [];
});

const selectedFilters = computed(() => {
  if (props.header.id === "kind") return props.filters.kindFilter;
  else if (props.header.id === "service") return props.filters.serviceFilter;
  else if (props.header.id === "changeSetName")
    return props.filters.changeSetFilter;
  else if (props.header.id === "actorName") return props.filters.userFilter;
  else return [];
});

const headerText = computed(() => {
  if (label.value === "Timestamp") {
    return `Sorting By Timestamp ${
      props.filters.sortTimestampAscending ? "(Oldest)" : "(Newest)"
    }`;
  }
  if (selectedFilters.value.length > 0) {
    return `Filtering by ${selectedFilters.value.length} selection${
      selectedFilters.value.length > 1 ? "s" : ""
    }`;
  } else return `Filter by ${label.value}`;
});

const headerTooltip = computed(() => {
  if (props.header.id === "ip") return null;

  return {
    content: headerText.value,
    delay: { show: 0, hide: 100 },
    instantMove: true,
  };
});

const dropdownMenuItems = computed(() => {
  const items: DropdownMenuItemObjectDef[] = [];

  items.push({
    label: headerText.value,
    header: true,
  });

  for (const k of filterOptions.value) {
    items.push({
      label: k.label,
      checkable: true,
      checked: selectedFilters.value.includes(k.value),
      onSelect: () => {
        emit("toggleFilter", k.value);
      },
    });
  }

  if (selectedFilters.value.length > 0) {
    items.unshift({
      label: "Clear Filters",
      onSelect: () => {
        emit("clearFilters");
      },
    });
  }

  return items;
});

const onClick = () => {
  if (props.header.id !== "ip") {
    dropdownMenuRef.value?.open();
  }
  emit("select");
};

const active = ref(false);

const startActive = () => {
  active.value = true;
  iconButtonRef.value?.startActive();
};

const endActive = () => {
  active.value = false;
  iconButtonRef.value?.endActive();
};

const emit = defineEmits<{
  (e: "select"): void;
  (e: "clearFilters"): void;
  (e: "toggleFilter", v: string): void;
}>();
</script>
