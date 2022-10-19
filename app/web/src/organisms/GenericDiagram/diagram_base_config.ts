/* eslint-disable @typescript-eslint/no-explicit-any, import/extensions */
import CheckIcon from "~icons/mdi/check?raw";
import CheckCircleIcon from "~icons/mdi/check-circle?raw";
import CheckSquareIcon from "~icons/mdi/check-box?raw";

import AlertIcon from "~icons/mdi/exclamation?raw";
import AlertCircleIcon from "~icons/mdi/alert-circle?raw";
import AlertSquareIcon from "~icons/mdi/alert-box?raw";

import ErrorIcon from "~icons/mdi/close?raw";
import ErrorCircleIcon from "~icons/mdi/close-circle?raw";
import ErrorSquareIcon from "~icons/mdi/close-box?raw";

import LoadingIcon from "~icons/gg/spinner?raw";

import MinusIcon from "~icons/mdi/minus?raw";

import { DiagramConfig } from "./diagram_types";
import { colors } from "../../utils/design_token_values";

export const baseConfig: DiagramConfig = {
  icons: {
    check: CheckIcon as any,
    "check-circle": CheckCircleIcon as any,
    "check-square": CheckSquareIcon as any,

    alert: AlertIcon as any,
    "alert-circle": AlertCircleIcon as any,
    "alert-square": AlertSquareIcon as any,

    error: ErrorIcon as any,
    "error-circle": ErrorCircleIcon as any,
    "error-square": ErrorSquareIcon as any,

    minus: MinusIcon as any,

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
