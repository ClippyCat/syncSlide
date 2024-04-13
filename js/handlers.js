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
  const markdownInput = document.getElementById("markdown-input");
  const mkInputAr = markdownInput.value.replaceAll("ARS", "<span lang=\"ar\" dir=\"rtl\">")
  const mkInputAr2 = mkInputAr.replaceAll("ARE", "</span>");
  const htmlOutput = document.getElementById("currentSlide");
  const newHtml = md.render(mkInputAr2);
  htmlOutput.innerHTML = newHtml;
  updateRender();
}
