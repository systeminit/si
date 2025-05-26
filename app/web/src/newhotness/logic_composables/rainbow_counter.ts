import { computed, reactive } from "vue";

const queue = reactive(new Set<string>());

/**
 * This is a global "stuff is happening" counter
 * When its > 0 the system is waiting for data
 */
export const rainbowCount = computed(() => queue.size);

export const add = (desc: string) => queue.add(desc);
export const remove = (desc: string) => queue.delete(desc);
