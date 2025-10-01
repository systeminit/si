// Extend a type, allowing overriding with compatible types
export type Extend<T, F> = Omit<T, keyof F> & F;
