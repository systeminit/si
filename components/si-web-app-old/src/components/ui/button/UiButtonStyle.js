export class UiButton {
  constructor(kind) {
    this.kind = kind;
  }

  get style() {
    var style;
    switch (this.kind) {
      case "icon":
        style =
          "inline-flex items-center transition-colors duration-300 ease-in focus:outline-none text-white hover:text-blue-400 focus:text-blue-100 rounded-l-full px-4 py-2 active";
        break;

      default:
        style =
          "bg-red-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded";
        break;
    }
    return style;
  }
}
