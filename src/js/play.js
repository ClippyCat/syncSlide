const video = document.getElementById('myVideo');
const rate = document.getElementById('rate');
const slidesContainer = document.getElementById('currentSlide');

let ACTIVE_CONTENT_IDX = -1;
video.ontimeupdate = (event) => {
	const newActiveIndex = slidesData.findLastIndex((sd) => sd.time <= video.currentTime);
	if (newActiveIndex !== -1 && newActiveIndex !== ACTIVE_CONTENT_IDX) {
		ACTIVE_CONTENT_IDX = newActiveIndex;
		slidesContainer.innerHTML = slidesData[ACTIVE_CONTENT_IDX].content;
	}
};

rate.onchange = function() {
video.playbackRate = rate.value;
};