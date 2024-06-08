const WEBSOCKET_ADDR = "wss://syncslide.clippycat.ca/ws/";
const socket = new WebSocket(WEBSOCKET_ADDR);
const md = new remarkable.Remarkable({
	html: true,
});

const updateRender = async () => {
	const htmlDiv = document.getElementById("currentSlide");
	renderMathInElement(htmlDiv, {
		delimiters: [
			{left: "$$", right: "$$", display: true},
			{left: "$", right: "$", display: false}
		],
		throwError: false,
	});
}

function stringToDOM(htmlString) {
	var tempElement = document.createElement('div');
	tempElement.innerHTML = htmlString.trim();
return tempElement.firstChild;
}

function getH2s(allHtml) {
	const h2s = allHtml.querySelectorAll('h2');
	const result = [];
	h2s.forEach(h2 => {
		const siblings = [];
		let sibling = h2.nextElementSibling;
		while (sibling && sibling.tagName !== 'H2') {
			siblings.push(sibling);
			sibling = sibling.nextElementSibling;
		}

		result.push({ h2, siblings });
	});
	return result;
}

const handleUpdate = (message) => {
	const htmlString = md.render(message.data);
	allHtml = stringToDOM(htmlString);
	slideIndex = document.getElementById("goTo").value;
	newHtml = getH2s(allHtml)[slideIndex-1];
	const htmlOutput = document.getElementById("currentSlide");
	htmlOutput.innerHTML = newHtml;
	updateRender();
}

const renderHTML = async () => {
	const markdownInput = document.getElementById("markdown-input").value;
	socket.send(markdownInput);
}

update = document.getElementById("update");
update.addEventListener("click", renderHTML);
socket.onmessage = handleUpdate
