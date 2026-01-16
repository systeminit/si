export class DefaultMap<K, V> extends Map<K, V> {
  constructor(private getDefaultValue: (key: K) => V, entries?: readonly (readonly [K, V])[] | null) {
    super(entries);
  }

  get = (key: K): V => {
    if (!this.has(key)) {
      this.set(key, this.getDefaultValue(key));
    }

    return super.get(key)!;
  };
}
