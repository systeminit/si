declare module 'searchjs' {
  interface Defaults {
    negator?: boolean;
    join?: string;
    text?: boolean;
    word?: boolean;
    separator?: string;
    propertySearch?: boolean;
    propertySearchDepth?: number;
    start?: boolean;
    end?: boolean;
  }

  type SingleField = null | undefined | boolean | number | Date | string;
  type Field = SingleField | SingleField[];

  interface SearchPrimitiveRange {
    from?: number | string;
    gte?: number | string;
    gt?: number | string;
    to?: number | string;
    lte?: number | string;
    lt?: number | string;
  }
  type SearchPrimitiveValues = null | undefined | string | number | Date | boolean | SearchPrimitiveRange;

  export type JoinValue = "OR" | "AND";

  type SearchPrimitive = {
    [key: string]: SearchPrimitiveValues | SearchPrimitiveValues[] | SearchPrimitive | SearchPrimitive[]
    terms?: SearchPrimitive[];
    _propertySearch?: boolean;
    _propertySearchDepth?: number;
    _not?: boolean | null;
    _join?: JoinValue;
    _text?: boolean;
    _word?: boolean;
    _start?: boolean;
    _end?: boolean;
    _separator?: string;
  }

  interface SearchObject  {
    [key: string]: Field | SearchObject
  }

  type SearchArray = SearchObject[] | undefined[];
  type SearchData = SearchObject;

  export function setDefaults(defaults: Defaults): void;
  export function resetDefaults(): void;
  export function singleMatch(field: Field, 
                       s: SearchPrimitive, 
                       text: Defaults["text"], 
                       word: Defaults["word"], 
                       start: Defaults["start"], 
                       end: Defaults["end"]): boolean;
  export function matchArray<T>(ary: T[], search: SearchPrimitive): T[];
  export function matchObject<T>(object: T, search: SearchPrimitive): T;
}
