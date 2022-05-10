import {
  EditField,
  EditFieldDataType,
  EditFields,
  VisibilityDiff,
} from "@/api/sdf/dal/edit_field";

/**
 * An interface for a [Visitor] over `EditField`s
 *
 * [Visitor]: https://en.wikipedia.org/wiki/Visitor_pattern
 */
export interface EditFieldVisitor {
  /**
   * Visits each `EditField` in the Array
   */
  visitEditFields(editFields: EditFields): void;

  /**
   * Visits an `EditField` which is an `Array`
   */
  visitArray(field: ArrayEditField): void;
  /**
   * Visits an `EditField` which is a `Boolean`
   */
  visitBoolean(field: BooleanEditField): void;
  /**
   * Visits an `EditField` which is an `Integer`
   */
  visitInteger(field: IntegerEditField): void;
  /**
   * Visits an `EditField` which is a `Map`
   */
  visitMap(field: MapEditField): void;
  /**
   * Visits an `EditField` which is a `None`
   */
  visitNone(field: NoneEditField): void;
  /**
   * Visits an `EditField` which is an `Object`
   */
  visitObject(field: ObjectEditField): void;
  /**
   * Visits an `EditField` which is a `String`
   */
  visitString(field: StringEditField): void;
}

/**
 * A Visitor which counts the number of changed edit fields
 */
export class ChangedEditFieldCounterVisitor implements EditFieldVisitor {
  private n = 0;

  count(): number {
    return this.n;
  }

  private countIfChanged(diff: VisibilityDiff) {
    if (diff.kind != "None") {
      this.n += 1;
    }
  }

  visitEditFields(editFields: EditFields) {
    for (const editField of editFields) {
      visitEditField(this, editField);
    }
  }

  visitArray(field: ArrayEditField) {
    this.countIfChanged(field.visibility_diff);

    if (field.widget.kind == "Array") {
      for (const entry of field.widget.options.entries) {
        visitEditField(this, entry);
      }
    } else {
      throw new Error(
        `Invalid Widget for an Array EditField: '${field.widget.kind}`,
      );
    }
  }

  visitBoolean(field: BooleanEditField) {
    this.countIfChanged(field.visibility_diff);
  }

  visitInteger(field: IntegerEditField) {
    this.countIfChanged(field.visibility_diff);
  }

  visitMap(field: MapEditField) {
    this.countIfChanged(field.visibility_diff);

    if (field.widget.kind == "Map") {
      for (const entry of field.widget.options.entries) {
        visitEditField(this, entry);
      }
    } else {
      throw new Error(
        `Invalid Widget for a Map EditField: '${field.widget.kind}`,
      );
    }
  }

  visitNone(field: NoneEditField) {
    this.countIfChanged(field.visibility_diff);
  }

  visitObject(field: ObjectEditField) {
    this.countIfChanged(field.visibility_diff);

    if (field.widget.kind == "Header") {
      for (const editField of field.widget.options.edit_fields) {
        visitEditField(this, editField);
      }
    } else {
      throw new Error(
        `Invalid Widget for an Object EditField: '${field.widget.kind}`,
      );
    }
  }

  visitString(field: StringEditField) {
    // Horrible hack to ensure the initial name setting isn't counted
    // We don't have enough metadata to be sure name is the actual root.si.name, so it might conflict eventually
    if (field.name === "name" && String(field.value).match(/^si-\d+$/)) {
      return;
    }
    this.countIfChanged(field.visibility_diff);
  }
}

export type ITreeOpenState = Record<string, boolean>;

export class InitialTreeOpenStateVisitor implements EditFieldVisitor {
  private state: ITreeOpenState = {};
  private currentFieldId: string | undefined = undefined;

  initialTreeState(): ITreeOpenState {
    return this.state;
  }

  private markOpenIfChildIsSet(field: EditField) {
    if (this.currentFieldId && field.value) {
      if (this.state[this.currentFieldId]) {
        this.state[this.currentFieldId] = true;
      } else {
        throw new Error(
          `Entry for header '${this.currentFieldId}' not set, this is a bug!`,
        );
      }
    }
  }

  visitEditFields(editFields: EditFields) {
    for (const editField of editFields) {
      visitEditField(this, editField);
    }
  }

  visitArray(field: ArrayEditField) {
    if (field.widget.kind == "Array") {
      for (const entry of field.widget.options.entries) {
        visitEditField(this, entry);
      }
    } else {
      throw new Error(
        `Invalid Widget for an Array EditField: '${field.widget.kind}`,
      );
    }
  }

  visitBoolean(field: BooleanEditField) {
    this.markOpenIfChildIsSet(field);
  }

  visitInteger(field: IntegerEditField) {
    this.markOpenIfChildIsSet(field);
  }

  visitMap(field: MapEditField) {
    if (field.widget.kind == "Map") {
      for (const entry of field.widget.options.entries) {
        visitEditField(this, entry);
      }
    } else {
      throw new Error(
        `Invalid Widget for a Map EditField: '${field.widget.kind}`,
      );
    }
  }

  visitNone(field: NoneEditField) {
    this.markOpenIfChildIsSet(field);
  }

  visitObject(field: ObjectEditField) {
    const headerId = field.id;
    this.state[headerId] = false;
    this.currentFieldId = headerId;

    if (field.widget.kind == "Header") {
      for (const editField of field.widget.options.edit_fields) {
        visitEditField(this, editField);
      }
    } else {
      throw new Error(
        `Invalid Widget for an Object EditField: '${field.widget.kind}`,
      );
    }

    this.currentFieldId = undefined;
  }

  visitString(field: StringEditField) {
    this.markOpenIfChildIsSet(field);
  }
}

interface VisitorField {
  accept(visitor: EditFieldVisitor): void;
}

export interface ArrayEditField extends EditField {
  data_type: EditFieldDataType.Array;
}

class ArrayVisitorField implements VisitorField {
  private editField: ArrayEditField;

  constructor(editField: ArrayEditField) {
    this.editField = editField;
  }

  accept(visitor: EditFieldVisitor) {
    visitor.visitArray(this.editField);
  }
}

export interface BooleanEditField extends EditField {
  data_type: EditFieldDataType.Boolean;
}

class BooleanVisitorField implements VisitorField {
  private editField: BooleanEditField;

  constructor(editField: BooleanEditField) {
    this.editField = editField;
  }

  accept(visitor: EditFieldVisitor) {
    visitor.visitBoolean(this.editField);
  }
}

export interface IntegerEditField extends EditField {
  data_type: EditFieldDataType.Integer;
}

class IntegerVisitorField implements VisitorField {
  private editField: IntegerEditField;

  constructor(editField: IntegerEditField) {
    this.editField = editField;
  }

  accept(visitor: EditFieldVisitor) {
    visitor.visitInteger(this.editField);
  }
}

export interface MapEditField extends EditField {
  data_type: EditFieldDataType.Map;
}

class MapVisitorField implements VisitorField {
  private editField: MapEditField;

  constructor(editField: MapEditField) {
    this.editField = editField;
  }

  accept(visitor: EditFieldVisitor) {
    visitor.visitMap(this.editField);
  }
}

export interface NoneEditField extends EditField {
  data_type: EditFieldDataType.None;
}

class NoneVisitorField implements VisitorField {
  private editField: NoneEditField;

  constructor(editField: NoneEditField) {
    this.editField = editField;
  }

  accept(visitor: EditFieldVisitor) {
    visitor.visitNone(this.editField);
  }
}

export interface ObjectEditField extends EditField {
  data_type: EditFieldDataType.Object;
}

class ObjectVisitorField implements VisitorField {
  private editField: ObjectEditField;

  constructor(editField: ObjectEditField) {
    this.editField = editField;
  }

  accept(visitor: EditFieldVisitor) {
    visitor.visitObject(this.editField);
  }
}

export interface StringEditField extends EditField {
  data_type: EditFieldDataType.String;
}

class StringVisitorField implements VisitorField {
  private editField: StringEditField;

  constructor(editField: StringEditField) {
    this.editField = editField;
  }

  accept(visitor: EditFieldVisitor) {
    visitor.visitString(this.editField);
  }
}

function visitEditField(visitor: EditFieldVisitor, editField: EditField) {
  switch (editField.data_type) {
    case EditFieldDataType.Array:
      new ArrayVisitorField(editField as ArrayEditField).accept(visitor);
      break;
    case EditFieldDataType.Boolean:
      new BooleanVisitorField(editField as BooleanEditField).accept(visitor);
      break;
    case EditFieldDataType.Integer:
      new IntegerVisitorField(editField as IntegerEditField).accept(visitor);
      break;
    case EditFieldDataType.Map:
      new MapVisitorField(editField as MapEditField).accept(visitor);
      break;
    case EditFieldDataType.None:
      new NoneVisitorField(editField as NoneEditField).accept(visitor);
      break;
    case EditFieldDataType.Object:
      new ObjectVisitorField(editField as ObjectEditField).accept(visitor);
      break;
    case EditFieldDataType.String:
      new StringVisitorField(editField as StringEditField).accept(visitor);
      break;
    default:
      throw new Error(`Unknown EditFieldDataType: ${editField.data_type}`);
  }
}
