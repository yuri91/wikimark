let base_url = window.location.origin+"/";

let simplemde = new SimpleMDE();

let uploading = false;

window.addEventListener("beforeunload", function(ev) {
	if (uploading)
		return;
	ev.preventDefault();
	ev.returnValue = "Are you sure to exit the editor? All unsaved changes will be lost";
});

function download() {
	let params = (new URL(document.location)).searchParams;
	let page = params.get("page");
	if (!page)
		return;
	fetch(base_url+"repo/"+page)
		.then(r => r.json())
		.then(md => {
		simplemde.value(md.content);
		document.getElementById("title").value = md.meta.title;
		document.getElementById("private").checked = md.meta.private;
	}).catch(e => console.log(e));
}
download();

function upload() {
	let params = (new URL(document.location)).searchParams;
	let title = document.getElementById("title").value;
	let private = document.getElementById("private").checked;
	if (!title) {
		console.log("empty title not allowed");
		return;
	}
	let content = simplemde.value();
	let data =  {
		title: title,
		content: content,
		private: private,
	};
	uploading = true;
	fetch(base_url+"commit", {
		method: "POST",
		body: JSON.stringify(data),
		headers: {
			'content-type' : 'application/json', 
		},
		credentials: 'include',
	}).then(r => {
		if (r.ok)
			return r.text();
		throw new Error("Response code is "+r.status);
	}).then(link => {
		window.location.href = base_url+'page/'+link;
	}).catch(e => console.log(e))
	.finally(() => { uploading = false; });
}
