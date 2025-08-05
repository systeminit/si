import { tw } from "@si/vue-lib";
import { IconNames } from "@si/vue-lib/design-system";
import { ActionKind } from "@/api/sdf/dal/action";

export const actionKindToAbbreviation = (actionKind: ActionKind) => {
  return {
    Create: "CRT",
    Destroy: "DLT",
    Refresh: "RFH",
    Manual: "MNL",
    Update: "UPT",
  }[actionKind];
};

export const actionIconClass = (kind: ActionKind) => {
  return {
    Create: tw`text-success-600`,
    Destroy: tw`text-destructive-500 dark:text-destructive-600`,
    Refresh: tw`text-action-600`,
    Manual: tw`text-action-600`,
    Update: tw`text-warning-600`,
  }[kind];
};

export const actionIcon = (kind: ActionKind) => {
  return {
    Create: "plus",
    Destroy: "trash",
    Refresh: "refresh",
    Manual: "play",
    Update: "tilde",
  }[kind] as IconNames;
};
