export interface LabelEntry<V> {
  label: string;
  value: V;
}

export type LabelList<V> = Array<LabelEntry<V>>;
