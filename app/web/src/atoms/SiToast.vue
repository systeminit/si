<template>
  <div class="absolute w-full h-full pointer-events-none m-0 top-0 left-0">
    <div class="flex flex-row-reverse p-5 h-full w-full m-0">
      <div class="flex flex-col-reverse m-0">
        <div
          v-for="toast in toasted"
          id="toast-success"
          :key="toast.id"
          class="flex items-center w-full max-w-xs p-4 mb-4 text-white-500 bg-[#434647] rounded-lg shadow pointer-events-auto"
          role="alert"
        >
          <div
            class="inline-flex items-center justify-center flex-shrink-0 w-8 h-8 rounded-lg"
            :class="
              toast.success
                ? ['text-green-500', 'bg-green-100']
                : ['text-red-500', 'bg-red-100']
            "
          >
            <CheckIcon v-if="toast.success" />
            <XIcon v-else />
          </div>
          <span class="mx-3 text-sm font-normal">
            <h3>
              <strong class="break-all">{{ toast.title }}</strong>
            </h3>
            <strong v-if="toast.subtitle" class="break-all">
              {{ toast.subtitle }}
            </strong>
            <p class="break-all">{{ message(toast) }}</p>
            <a
              v-if="toast.message.length > 100"
              class="cursor-pointer underline w-auto"
              @click="toast.viewFullOutput = !toast.viewFullOutput"
            >
              <span v-if="toast.viewFullOutput">hide full output</span>
              <span v-else>view full output</span>
            </a>
          </span>
          <button
            type="button"
            class="ml-auto -mx-1.5 -my-1.5 bg-[#434647] text-gray-400 hover:text-gray-900 rounded-lg focus:ring-2 focus:ring-gray-300 p-1.5 hover:bg-gray-100 inline-flex h-8 w-8"
            aria-label="Close"
            @click="hideToasted(toast.id)"
          >
            <span class="sr-only">Close</span>
            <XIcon class="w-5 h-5" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { XIcon, CheckIcon } from "@heroicons/vue/solid";
import { toast$, Toasted } from "@/observable/toast";
import { ref } from "vue";
import { untilUnmounted } from "vuse-rx";

interface Burnt extends Toasted {
  timeout: ReturnType<typeof setTimeout>;
  viewFullOutput: boolean;
}

const toasted = ref<Burnt[]>([]);

const hideToasted = (id: string) => {
  const old = toasted.value.find((t) => t.id === id);
  if (old) clearTimeout(old.timeout);

  toasted.value = toasted.value.filter((t) => t.id !== id);
};

const message = (toast: Burnt) => {
  if (toast.viewFullOutput) return toast.message;

  const maxSize = 100;
  return (
    toast.message.substring(0, maxSize) +
    (toast.message.length > maxSize ? "..." : "")
  );
};

toast$.pipe(untilUnmounted).subscribe((toast) => {
  if (!toast) return;

  hideToasted(toast.id);

  const maxSize = 100;
  const toaster = {
    timeout: setTimeout(() => hideToasted(toast.id), 8000),
    viewFullOutput: toast.message.length <= maxSize,
    ...toast,
  };
  toasted.value.push(toaster);
});
</script>
