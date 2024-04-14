const renderMathInElement = require('katex/contrib/auto-render/auto-render');

function updateRender(html) {
    const htmlDiv = document.getElementById("currentSlide");
    htmlDiv.innerHTML = html;
    renderMathInElement(htmlDiv, {
        delimiters: [
            {left: "$$", right: "$$", display: true},
            {left: "$", right: "$", display: false}
        ],
        throwError: false,
    });
}

function handleMessage(event) {
    const markdown = event.data;
    const md = new remarkable.Remarkable({ html: true });
    const html = md.render(markdown);
    updateRender(html);
}

const socket = new WebSocket('ws://localhost:8080');

socket.onmessage = handleMessage;
