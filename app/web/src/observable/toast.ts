import { ReplaySubject } from "rxjs";

export interface Toasted {
  id: string;
  success: boolean;
  title: string;
  message: string;
}

export const toast$ = new ReplaySubject<Toasted>(1);
