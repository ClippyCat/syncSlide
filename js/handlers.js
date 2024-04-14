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

const renderHTML = async () => {
	const md = new remarkable.Remarkable({
		html: true,
	});
	const markdownInput = document.getElementById("markdown-input").value;
	const htmlOutput = document.getElementById("currentSlide");
	getSlide = markdownInput.split("\n## ");
	numSlides = getSlide.length;
	goto = document.getElementById("goTo");
	goto.max = numSlides;
slideIndex = goto.value;

	const newHtml = md.render("## "+getSlide[slideIndex]);
	htmlOutput.innerHTML = newHtml;
	updateRender();
}


update = document.getElementById("update");
update.addEventListener("click", renderHTML);
