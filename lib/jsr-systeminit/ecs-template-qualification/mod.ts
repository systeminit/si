/**
 * Code to help validate ECS clusters
 *
 * Usage:
 *
 * ```
 * import { isValidFargateCpuMemoryCombination } from
 * "jsr:@systeminit/ecs-template-qualification";
 *
 * isValidFargateCpuMemoryCombination(256, 512); # true
 * isValidFargateCpuMemoryCombination(256, 8192); # false
 * ```
 * @module
 */

/*
 * CPU/Memory combination pairs
 */
export type CpuMemoryPair = {
  cpu: number;
  memory: number[];
};

/*
 * The valid combinations of CPU and Memory
 */
export const validCpuMemoryCombinations: CpuMemoryPair[] = [
  { cpu: 256, memory: [512, 1024, 2048] }, // 0.25 vCPU
  { cpu: 512, memory: [1024, 2048, 3072, 4096] }, // 0.5 vCPU
  { cpu: 1024, memory: [2048, 3072, 4096, 5120, 6144, 7168, 8192] }, // 1 vCPU
  { cpu: 2048, memory: Array.from({ length: 13 }, (_, i) => 4096 + i * 1024) }, // 2 vCPU: 4GB to 16GB in 1GB increments
  { cpu: 4096, memory: Array.from({ length: 23 }, (_, i) => 8192 + i * 1024) }, // 4 vCPU: 8GB to 30GB in 1GB increments
  { cpu: 8192, memory: Array.from({ length: 12 }, (_, i) => 32768 + i * 8192) }, // 8 vCPU: 32GB to 120GB in 8GB increments
  {
    cpu: 16384,
    memory: Array.from({ length: 12 }, (_, i) => 65536 + i * 8192),
  }, // 16 vCPU: 64GB to 192GB in 8GB increments
];

/*
 * Takes a number of CPU and a number of memory, and validates that they are
 * both correct. Memory must be specified in MiB.
 */
export function isValidFargateCpuMemoryCombination(
  cpu: number,
  memory: number,
): boolean {
  const combination = validCpuMemoryCombinations.find((c) => c.cpu === cpu);
  return combination ? combination.memory.includes(memory) : false;
}

/*
 * Print the valid combinations of CPU and Memory
 */
export function printValidCombos() {
  for (const { cpu, memory } of validCpuMemoryCombinations) {
    console.log("-----------");
    console.log(`CPU: ${cpu}`);
    console.log("-----------");
    for (const mem of memory) {
      console.log(`Memory: ${mem}`);
    }
    console.log("");
  }
}
