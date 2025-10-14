import { PROVIDER_REGISTRY } from "./pipelines/types.ts";

export type Provider = keyof typeof PROVIDER_REGISTRY | "all";
