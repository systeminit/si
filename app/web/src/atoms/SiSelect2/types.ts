export interface SiSelectOption {
  text: string;
  action: (() => Promise<void>) | (() => void);
}
