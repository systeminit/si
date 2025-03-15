/**
 * Type utility for extending TypeScript types.
 * 
 * This module provides a utility type for extending interfaces in a type-safe way,
 * ensuring that the overridden types extend the original.
 * 
 * @module extend
 */

/**
 * Extends a type T with new fields F, ensuring type safety.
 * 
 * This is particularly useful when extending JSON Schema types while
 * maintaining type compatibility.
 * 
 * @template T The base type to extend
 * @template F The new fields to add (must be a partial of T)
 */
export type Extend<T, F extends Partial<T>> = Omit<T, keyof F> & F;
