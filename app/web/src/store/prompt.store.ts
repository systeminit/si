import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { ref, watch, watchEffect } from "vue";
import { useRealtimeStore } from "./realtime/realtime.store";

export type PromptKind = string;

export interface PromptEntry {
  kind: PromptKind;
  overridden: boolean;
}

const PROMPTS_API = "v2/admin/prompts";

export function usePromptStore() {
  return addStoreHooks(
    null,
    null,
    defineStore(`wsNONE/admin/prompts`, {
      state: () => ({
        prompts: new Array<PromptEntry>(),
        selectedPromptKind: ref<PromptKind>(),
        selectedPromptText: "",
      }),
      getters: {
        selectedPromptOverridden: ({ prompts, selectedPromptKind }) =>
          prompts.find((p) => p.kind === selectedPromptKind)?.overridden,
      },
      actions: {
        async FETCH_PROMPT_KINDS() {
          return new ApiRequest({
            url: PROMPTS_API,
            onSuccess: (prompts: PromptEntry[]) => {
              this.prompts = prompts;
            },
          });
        },
        async FETCH_PROMPT(kind: PromptKind) {
          return new ApiRequest<PromptEntry & { prompt_yaml: string }>({
            url: `${PROMPTS_API}/${kind}`,
            keyRequestStatusBy: kind,
            onSuccess: ({ prompt_yaml }) => {
              if (kind === this.selectedPromptKind)
                this.selectedPromptText = prompt_yaml;
            },
          });
        },
        async OVERRIDE_PROMPT(kind: PromptKind, text: string) {
          return new ApiRequest({
            method: "put",
            url: `${PROMPTS_API}/${kind}`,
            keyRequestStatusBy: kind,
            params: { prompt_yaml: text },
          });
        },
        async RESET_PROMPT(kind: PromptKind) {
          return new ApiRequest<PromptEntry>({
            method: "delete",
            url: `${PROMPTS_API}/${kind}`,
            keyRequestStatusBy: kind,
            onSuccess: () => {
              // Update the prompt on reset
              if (kind === this.selectedPromptKind) this.FETCH_PROMPT(kind);
            },
          });
        },
      },
      onActivated() {
        this.FETCH_PROMPT_KINDS();

        // Default selectedPromptKind to the first one (if available)
        // TODO this is almost a computed value, except it needs to be settable as well to be
        // used in a v-model. There must be a better way to do that.
        watchEffect(() => {
          if (undefined === this.selectedPromptKind)
            this.selectedPromptKind = this.prompts[0]?.kind;
        });

        // Fetch selectedPromptText when selectedPromptKind changes
        watch(
          () => this.selectedPromptKind,
          (kind) => {
            this.selectedPromptText = "";
            if (kind) this.FETCH_PROMPT(kind);
          },
        );

        // Listen to events on the current change set.
        const realtimeStore = useRealtimeStore();
        realtimeStore.subscribe(this.$id, "all", [
          {
            eventType: "PromptUpdated",
            callback: ({ kind, overridden }) => {
              const prompt = this.prompts.find((p) => p.kind === kind);
              if (prompt) prompt.overridden = overridden;
              // TODO this is how you make prompt updates multiplayer, but we don't want to
              // overwrite prompts that the user is editing, so we need to decide how to
              // handle it.
              // if (kind === this.selectedPromptKind) this.FETCH_PROMPT(kind);
            },
          },
        ]);
      },
    }),
  )();
}
