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

const handleUpdate = (message) => {
	const newHtml = md.render(message.data);
	const htmlOutput = document.getElementById("currentSlide");
	htmlOutput.innerHTML = newHtml;
	updateRender();
}

socket.onmessage = handleUpdate
