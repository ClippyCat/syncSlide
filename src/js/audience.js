const WEBSOCKET_ADDR = "ws://172.81.178.147:5000/";
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
