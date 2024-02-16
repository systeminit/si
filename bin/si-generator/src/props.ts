export interface PropBase {
  kind: string;
  name: string;
  variableName: string;
}

export interface PropString extends PropBase {
  kind: "string";
}

export interface PropNumber extends PropBase {
  kind: "number";
}

export interface PropBoolean extends PropBase {
  kind: "boolean";
}

export interface PropObject extends PropBase {
  kind: "object";
  children: Array<Prop>;
}

export interface PropMap extends PropBase {
  kind: "map";
  entry?: Prop;
}

export interface PropArray extends PropBase {
  kind: "array";
  entry?: Prop;
}

export type Prop =
  | PropString
  | PropNumber
  | PropObject
  | PropArray
  | PropBoolean
  | PropMap;

export type PropParent = PropObject | PropArray | PropMap;
