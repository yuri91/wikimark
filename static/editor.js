import Alpine from "https://cdn.jsdelivr.net/npm/alpinejs@3.13.3/dist/module.esm.js";
async function require(path) {
	let _module = window.module;
	let _exports = window.exports;
	window.module = {};
	window.exports = {};
	await import(path);
	let exports = module.exports;
	window.module = _module; // restore global
	window.exports = _exports; // restore global
	return exports;
}
function load_css(url) {
  let el = document.createElement("link");
  el.href = url;
  el.rel = "stylesheet";
  document.head.appendChild(el);
}

load_css("https://cdn.jsdelivr.net/npm/simplemde@1.11.2/dist/simplemde.min.css");

let SimpleMDE = await require("https://cdn.jsdelivr.net/simplemde/latest/simplemde.min.js");

window.Alpine = Alpine;
Alpine.data("editor", () => ({
  mde: null,
  init() {
    let mde = new SimpleMDE({
      element: this.$el,
      autoDownloadFontAwesome: true,
    });
    mde.value(this.$el.text);
    this.mde = mde;

    // Keep textarea value synced for htmx form serialization
    mde.codemirror.on('change', () => {
      this.$el.value = mde.value();
    });
  },
}));
Alpine.start();
