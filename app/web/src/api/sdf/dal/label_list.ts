export interface LabelEntry<V> {
  label: String;
  value: V;
}

export type LabelList<V> = Array<LabelEntry<V>>;
