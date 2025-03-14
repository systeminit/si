import { assertEquals } from "@std/assert";
import {
  isValidFargateCpuMemoryCombination,
  validCpuMemoryCombinations,
} from "./mod.ts";

Deno.test(function validCombinations() {
  for (const { cpu, memory: memoryValues } of validCpuMemoryCombinations) {
    for (const memory of memoryValues) {
      assertEquals(
        isValidFargateCpuMemoryCombination(cpu, memory),
        true,
        `cpu: ${cpu}, memory: ${memory}`,
      );
    }
  }
  assertEquals(
    isValidFargateCpuMemoryCombination(16384, 122880),
    true,
    `cpu: 16384, memory: 122880`,
  );
});

Deno.test(function inValidCombinations() {
  for (const { cpu, memory: memoryValues } of validCpuMemoryCombinations) {
    for (const memory of memoryValues) {
      assertEquals(isValidFargateCpuMemoryCombination(cpu, memory + 1), false);
    }
  }
});
