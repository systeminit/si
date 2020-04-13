import { Prop } from "@/prop";
import { PropText } from "@/prop/text";

interface AddArguments<T extends Prop> {
  name: string;
  label: string;
  options?(p: T): void;
}

export function addText(addArgs: AddArguments<PropText>): void {
  const p = new PropText(addArgs);
  if (addArgs.options) {
    addArgs.options(p);
  }
}
