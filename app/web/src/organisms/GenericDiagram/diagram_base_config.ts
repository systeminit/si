import { DiagramConfig } from "./diagram_types";
import { colors } from "../../utils/design_token_values";

export const baseConfig: DiagramConfig = {
  toneColors: {
    success: colors.success[500],
    warning: colors.warning[500],
    destructive: colors.destructive[500],
    error: colors.destructive[500],
    action: colors.action[500],
    info: colors.action[500],
    neutral: colors.neutral[500],
  },
};
