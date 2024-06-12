const video = document.getElementById('myVideo');
const rate = document.getElementById('rate');
const slidesContainer = document.getElementById('currentSlide');
const slidesData = [{"time":0,"slide":"0","content":"<h1>Short Demo</h1><h2>Jeopardy</h2><p>In a studio where silence reigns,\nA battleground of wit remains,\nThree contenders stand their ground,\nTo claim the prize where smarts are crowned.</p>"},{"time":4,"slide":"1","content":"<h1>Short Demo</h1><h2>Jeopardy</h2><p>The host, with charming ease and grace,\nUnveils the board, each column's face,\nA grid of knowledge, vast and wide,\nWhere answers wait and truths abide.</p>"},{"time":8,"slide":"2","content":"<h1>Short Demo</h1><h2>Jeopardy</h2><p>From \"Potent Potables\" to \"World's Stage,\"\nEach clue invites a mental gauge,\nIn categories broad and grand,\nA test of intellect's demand.</p>"},{"time":12,"slide":"3","content":"<h1>Short Demo</h1><h2>Jeopardy</h2><p>With buzzing hearts and trembling hands,\nThey wager well and make their stands,\nEach question framed in perfect style,\nTo tempt the mind, provoke a smile.</p>"},{"time":16,"slide":"4","content":"<h1>Short Demo</h1><h2>Jeopardy</h2><p>Daily Doubles hide and tease,\nRisk it all or play with ease,\nStrategize with every call,\nTo rise above, or face the fall.</p>"},{"time":20,"slide":"5","content":"<h1>Short Demo</h1><h2>Jeopardy</h2><p>The Final Jeopardy, a silent quake,\nThe last great chance, the move to make,\nTo scribble answers, clear and bright,\nIn hopes to win, to claim the night.</p>"},{"time":24,"slide":"6","content":"<h1>Short Demo</h1><h2>Jeopardy</h2><p>Applause erupts, the scores revealed,\nA champion stands, the fate is sealed,\nIn Jeopardy's arena bright,\nThe quest for knowledge takes its flight.</p>"},{"time":28,"slide":"7","content":"<h1>Short Demo</h1><h2>Jeopardy</h2><p>Yet win or lose, each mind displayed,\nIn trivia's realm, where thoughts are laid,\nA tribute to the human brain,\nIn Alex's name, forever reign.</p>"}];

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