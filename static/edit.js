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
	console.log(page);
	if (page===undefined)
		return;
	fetch(base_url+"repo/"+page)
		.then(r => r.json())
		.then(md => {
		simplemde.value(md.content);
		document.getElementById("title").value = md.meta.title;
		document.getElementById("private").checked = md.meta.private;
		let comps = page.split("/");
		comps.pop();
		document.getElementById("dir").value = comps.join("/");
	}).catch(e => console.log(e));
}
download();

async function upload() {
	let params = (new URL(document.location)).searchParams;
	let title = document.getElementById("title").value;
	let private = document.getElementById("private").checked;
	let dir = document.getElementById("dir").value;
	if (!title) {
		console.log("empty title not allowed");
		return;
	}
	let content = simplemde.value();
	let data =  {
		page: {
			meta: {
				title: title,
				private: private,
			},
			content: content,
		},
		dir,
	};
	uploading = true;
	try {
		let resp = fetch(base_url+"commit", {
			method: "POST",
			body: JSON.stringify(data),
			headers: {
				'content-type' : 'application/json', 
			},
			credentials: 'include',
		});
		if (resp.ok) {
			let link = await resp.text();
			window.location.href = base_url+'page/'+link;
		}
	} catch (e) {
		console.log(e);
	}
	uploading = false;
}
