/* eslint-disable @typescript-eslint/no-explicit-any, import/extensions */
import CheckIcon from "~icons/material-symbols/check-circle?raw";
import AlertIcon from "~icons/material-symbols/warning?raw";
import ErrorIcon from "~icons/material-symbols/cancel?raw";
import LoadingIcon from "~icons/gg/spinner?raw";

import { DiagramConfig } from "./diagram_types";
import { colors } from "../../utils/design_token_values";

export const baseConfig: DiagramConfig = {
  icons: {
    check: CheckIcon as any,
    alert: AlertIcon as any,
    error: ErrorIcon as any,
    loading: LoadingIcon as any,
  },
  toneColors: {
    success: colors.success[500],
    warning: colors.warning[500],
    error: colors.destructive[500],
    info: colors.action[500],
    neutral: colors.neutral[500],
  },
};
