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
  Boolean = "boolean",
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
  booleanTerm?: BooleanTerm;
  isNot?: boolean;
  items: IItem[];
}

export class Query implements IQuery {
  booleanTerm?: IQuery["booleanTerm"];
  isNot?: IQuery["isNot"];
  items: IQuery["items"];

  constructor(args: IQuery) {
    this.booleanTerm = args.booleanTerm;
    this.isNot = args.isNot;
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
