const video = document.getElementById("myVideo");
const rate = document.getElementById("rate");
const slidesContainer = document.getElementById("currentSlide");
const goTo = document.getElementById("goTo");
const go = document.getElementById("go");

goTo.innerHTML = "<option value=0>Start: 0</option>";
slidesData.forEach((e, i) => {
	const newOption = document.createElement('option');
	newOption.value = e.time;
	newOption.innerText = e.title + ": " + e.time;
	goTo.appendChild(newOption);
});

let ACTIVE_CONTENT_IDX = -1;
video.ontimeupdate = (event) => {
	const newActiveIndex = slidesData.findLastIndex((sd) => sd.time <= video.currentTime);
	if (newActiveIndex !== -1 && newActiveIndex !== ACTIVE_CONTENT_IDX) {
		ACTIVE_CONTENT_IDX = newActiveIndex;
		slidesContainer.innerHTML = slidesData[ACTIVE_CONTENT_IDX].content;
		goTo.value = slidesData[ACTIVE_CONTENT_IDX].time;
	}
};

go.onclick = function() {
	video.currentTime = goTo.value
}

rate.onchange = function() {
video.playbackRate = rate.value;
};