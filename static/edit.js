let base_url = "http://localhost:8000/";

let simplemde = new SimpleMDE();

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
	}).catch(e => console.log(e));
}
download();

function upload() {
	let params = (new URL(document.location)).searchParams;
	let title = document.getElementById("title").value;
	let content = simplemde.value();
	let data =  {
		title: title,
		content: content,
	};
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
	}).catch(e => console.log(e));
}
