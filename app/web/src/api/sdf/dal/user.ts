export interface User {
  pk: string;
  name: string;
  email: string;
  // TODO should be camelcase, but changing backend is annoying
  picture_url?: string;
  created_at: IsoDateString;
  updated_at: IsoDateString;
}
