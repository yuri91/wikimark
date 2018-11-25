let base_url = "http://localhost:8000/";

let simplemde = new SimpleMDE();

let front_regex = /\+\+\+([\s\S]*)\+\+\+\n\n/;
function download() {
	let params = (new URL(document.location)).searchParams;
	let page = params.get("page");
	if (!page)
		return;
	fetch(base_url+"page/"+page)
		.then(r => r.json())
		.then(data => {
		if (data.error) {
			console.log(data);
			return;
		}
		simplemde.value(data.content);
		document.getElementById("title").value = data.metadata.title;
	}).catch(e => console.log(e));
}
download();

function upload() {
	let params = (new URL(document.location)).searchParams;
	let page = params.get("page");
	let title = document.getElementById("title").value;
	if (!page) {
		page = getSlug(title, {
			separator: '-',
		});
	}
	console.log(page);
	let content = simplemde.value();
	let data =  {
		metadata: {
			title: title, 
		},
		content: content,
	};
	fetch(base_url+"page/"+page, {
		method: "POST",
		body: JSON.stringify(data),
		headers: {
			'content-type' : 'application/json', 
		},
		credentials: 'include',
	}).then(r => {
		window.location.href = base_url+'pages/'+page;
	}).catch(e => console.log(e));
}
