// TODO: Your mission, if you choose to accept it - get query working, so that you can drive Workspace
// and Organization listing.

export enum OrderByDirection {
  ASC = "asc",
  DESC = "desc",
}

export enum BooleanTerm {
  And = "and",
  Or = "or",
}

export enum Comparison {
  Equals = "equals",
  NotEquals = "notEquals",
  Contains = "contains",
  Like = "like",
  NotLike = "notLike",
}

export enum FieldType {
  String = "string",
  Int = "int",
}

export interface IExpression {
  field: string;
  value: string;
  comparison: Comparison;
  fieldType: FieldType;
}

export interface IItem {
  query?: IQuery;
  expression?: IExpression;
}

export interface IQuery {
  boolean_term?: BooleanTerm;
  is_not?: boolean;
  items: IItem[];
}

export class Query implements IQuery {
  boolean_term?: IQuery["boolean_term"];
  is_not?: IQuery["is_not"];
  items: IQuery["items"];

  constructor(args: IQuery) {
    this.boolean_term = args.boolean_term;
    this.is_not = args.is_not;
    this.items = args.items;
  }

  static for_simple_string(
    field: IExpression["field"],
    value: IExpression["value"],
    comparison: IExpression["comparison"],
  ): Query {
    return new Query({
      items: [
        {
          expression: {
            field,
            value,
            comparison,
            fieldType: FieldType.String,
          },
        },
      ],
    });
  }
}
