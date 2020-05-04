export type Associations = BelongsTo;

interface AssociationConstructor {
  typeName: Association["typeName"];
  methodName: Association["methodName"];
  methodArgumentName: BelongsTo["methodArgumentName"];
  fieldName?: Association["fieldName"];
}

export class Association {
  typeName: string;
  methodName: string;
  methodArgumentName: string;
  fieldName: string;

  constructor(args: AssociationConstructor) {
    this.typeName = args.typeName;
    this.methodName = args.methodName;
    this.methodArgumentName = args.methodArgumentName;
    if (args.fieldName == undefined) {
      this.fieldName = args.typeName;
    } else {
      this.fieldName = args.fieldName;
    }
  }
}

interface BelongsToConstructor
  extends Omit<AssociationConstructor, "methodName" | "methodArgumentName"> {
  fromFieldPath: BelongsTo["fromFieldPath"];
  methodName?: Association["methodName"];
  methodArgumentName?: Association["methodArgumentName"];
}

export class BelongsTo extends Association {
  fromFieldPath: string[];

  constructor(args: BelongsToConstructor) {
    if (args.methodName == undefined) {
      args.methodName = "get";
    }
    if (args.methodArgumentName == undefined) {
      args.methodArgumentName = `id`;
    }
    super(args as AssociationConstructor);
    this.fromFieldPath = args.fromFieldPath;
  }

  kind(): string {
    return "belongsTo";
  }
}

interface HasManyConstructor
  extends Omit<
    AssociationConstructor,
    "methodName" | "methodArgumentName" | "fromFieldPath"
  > {
  fromFieldPath?: HasMany["fromFieldPath"];
  methodName?: Association["methodName"];
  methodArgumentName?: Association["methodArgumentName"];
}

export class HasMany extends Association {
  fromFieldPath: string[];

  constructor(args: HasManyConstructor) {
    if (args.methodName == undefined) {
      args.methodName = "list";
    }
    if (args.methodArgumentName == undefined) {
      args.methodArgumentName = `input`;
    }
    super(args as AssociationConstructor);
    if (args.fromFieldPath) {
      this.fromFieldPath = args.fromFieldPath;
    } else {
      this.fromFieldPath = ["id"];
    }
  }

  kind(): string {
    return "hasMany";
  }
}

interface HasListConstructor
  extends Omit<AssociationConstructor, "methodName" | "methodArgumentName"> {
  fromFieldPath: HasList["fromFieldPath"];
  methodName?: Association["methodName"];
  methodArgumentName?: Association["methodArgumentName"];
}

export class HasList extends Association {
  fromFieldPath: string[];

  constructor(args: HasListConstructor) {
    if (args.methodName == undefined) {
      args.methodName = "list";
    }
    if (args.methodArgumentName == undefined) {
      args.methodArgumentName = `input`;
    }
    super(args as AssociationConstructor);
    this.fromFieldPath = args.fromFieldPath;
  }

  kind(): string {
    return "hasList";
  }
}

interface InListConstructor
  extends Omit<
    AssociationConstructor,
    "methodName" | "methodArgumentName" | "fromFieldPath"
  > {
  toFieldPath: InList["toFieldPath"];
  fromFieldPath?: InList["fromFieldPath"];
  methodName?: Association["methodName"];
  methodArgumentName?: Association["methodArgumentName"];
}

export class InList extends Association {
  fromFieldPath: string[];
  toFieldPath: string[];

  constructor(args: InListConstructor) {
    if (args.methodName == undefined) {
      args.methodName = "list";
    }
    if (args.methodArgumentName == undefined) {
      args.methodArgumentName = `input`;
    }
    super(args as AssociationConstructor);
    if (args.fromFieldPath) {
      this.fromFieldPath = args.fromFieldPath;
    } else {
      this.fromFieldPath = ["id"];
    }
    this.toFieldPath = args.toFieldPath;
  }

  kind(): string {
    return "inList";
  }
}

export class AssociationList {
  associations: Associations[] = [];

  all(): AssociationList["associations"] {
    return this.associations;
  }

  getByFieldName(fieldName: string): Associations {
    const result = this.associations.find(a => a.fieldName == fieldName);
    if (result == undefined) {
      throw `Cannot get association field ${fieldName}; it does not exist on the object`;
    }
    return result;
  }

  getByTypeName(typeName: string): Associations {
    const result = this.associations.find(a => a.typeName == typeName);
    if (result == undefined) {
      throw `Cannot get association type ${typeName}; it does not exist on the object`;
    }
    return result;
  }

  belongsTo(args: BelongsToConstructor): BelongsTo {
    const assoc = new BelongsTo(args);
    this.associations.push(assoc);
    return assoc;
  }

  hasMany(args: HasManyConstructor): HasMany {
    const assoc = new HasMany(args);
    this.associations.push(assoc);
    return assoc;
  }

  hasList(args: HasListConstructor): HasMany {
    const assoc = new HasList(args);
    this.associations.push(assoc);
    return assoc;
  }

  inList(args: InListConstructor): HasMany {
    const assoc = new InList(args);
    this.associations.push(assoc);
    return assoc;
  }
}
