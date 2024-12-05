export interface LabelEntry<V> {
  label: string;
  value: V;
}

export interface DoubleLabelEntry<V> extends LabelEntry<V> {
  label2: string;
  componentId?: string;
}

export type LabelList<V> = Array<LabelEntry<V>>;

export type DoubleLabelList<V> = Array<DoubleLabelEntry<V>>;
