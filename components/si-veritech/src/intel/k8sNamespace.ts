import {
  baseInferProperties,
  baseCheckQualifications,
  baseRunCommands,
  baseSyncResource,
} from "./k8sShared";

export default {
  inferProperties: baseInferProperties,
  checkQualifications: baseCheckQualifications,
  runCommands: baseRunCommands,
  syncResource: baseSyncResource,
};
