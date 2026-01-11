import "./vendor/alpine.min.js";
import "./vendor/easymde.min.js";
import easyMDESheet from "./vendor/easymde.min.css" with { type: "css" };

document.adoptedStyleSheets = [easyMDESheet];

Alpine.data("editor", () => ({
  mde: null,
  init() {
    let mde = new EasyMDE({
      element: this.$el,
      autoDownloadFontAwesome: true,
      forceSync: true,
    });
    mde.value(this.$el.text);
    this.mde = mde;
  },
}));
