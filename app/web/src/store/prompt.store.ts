import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { defineStore, storeToRefs } from "pinia";
import * as _ from "lodash-es";
import { ref, watch } from "vue";

export type PromptKind = string;

export interface PromptEntry {
  kind: PromptKind;
  overridden: boolean;
}

const PROMPTS_API = "v2/admin/prompts";

export const usePromptStore = () => {
  return addStoreHooks(
    null,
    null,
    defineStore(`wsNONE/admin/prompts/selected`, {
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
              if (!this.selectedPromptKind)
                this.selectedPromptKind = prompts[0]?.kind;
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
            onSuccess: () => {
              const prompt = this.prompts.find((p) => p.kind === kind);
              if (prompt) prompt.overridden = true;
            },
          });
        },
        async RESET_PROMPT(kind: PromptKind) {
          return new ApiRequest<PromptEntry>({
            method: "delete",
            url: `${PROMPTS_API}/${kind}`,
            keyRequestStatusBy: kind,
            onSuccess: () => {
              const prompt = this.prompts.find((p) => p.kind === kind);
              if (prompt) prompt.overridden = false;
              // Update the prompt on reset
              if (kind) this.FETCH_PROMPT(kind);
            },
          });
        },
      },
      onActivated() {
        this.FETCH_PROMPT_KINDS();

        // Fetch selectedPromptText when selectedPromptKind changes
        const { selectedPromptKind } = storeToRefs(this);
        watch(selectedPromptKind, (kind) => {
          this.selectedPromptText = "";
          if (kind) this.FETCH_PROMPT(kind);
        });

        // TODO listen to WsEvents saying which things are and are not overridden
      },
    }),
  )();
};
