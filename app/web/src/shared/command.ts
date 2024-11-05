export type VoidFn = () => void;
export type FactoryFn = () => Command;

export type Option = { label: string; value: string };

export type CommandArg =
  | "component"
  | "outputSocket"
  | "inputSocket"
  | "schema"
  | "action";

export interface Command {
  readonly name: string;
  readonly shortcut: string;
  readonly expects: CommandArg[];
  choices: Option[];
  execute: VoidFn;
  factory: FactoryFn;
}
