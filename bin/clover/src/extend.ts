// Extend a type, ensuring that the overridden types extend the original
export type Extend<T, F extends Partial<T>> = Omit<T, keyof F> & F;
