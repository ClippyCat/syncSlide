const WEBSOCKET_ADDR = "ws://172.81.178.147:5002/";
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

const handleUpdate = (message) => {
	const newHtml = md.render(message.data);
	const htmlOutput = document.getElementById("currentSlide");
	htmlOutput.innerHTML = newHtml;
	updateRender();
}

const renderHTML = async () => {
	const markdownInput = document.getElementById("markdown-input").value;
	const title = markdownInput.split('\n')[0];
	getSlide = markdownInput.split("\n## ");
	numSlides = getSlide.length;
	goto = document.getElementById("goTo");
	goto.max = numSlides-1;
	slideIndex = goto.value;
	socket.send(title + "\n" + "## " + getSlide[slideIndex]);
}

update = document.getElementById("update");
update.addEventListener("click", renderHTML);
socket.onmessage = handleUpdate
