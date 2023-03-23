import { COLOR_PALETTE } from "@si/vue-lib/design-system";
import { DiagramConfig } from "./diagram_types";

export const baseConfig: DiagramConfig = {
  // TODO: pull in tones instead
  toneColors: {
    success: COLOR_PALETTE.success[500],
    warning: COLOR_PALETTE.warning[500],
    destructive: COLOR_PALETTE.destructive[500],
    error: COLOR_PALETTE.destructive[500],
    action: COLOR_PALETTE.action[500],
    info: COLOR_PALETTE.action[500],
    neutral: COLOR_PALETTE.neutral[500],
  },
};
