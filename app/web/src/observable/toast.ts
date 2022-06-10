import { ReplaySubject } from "rxjs";

export interface Toasted {
  id: string;
  success: boolean;
  title: string;
  subtitle?: string;
  message: string;
}

export const toast$ = new ReplaySubject<Toasted>(1);
