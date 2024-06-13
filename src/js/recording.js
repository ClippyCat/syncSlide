let recording = false;
let paused = false;
let startTime;
let elapsedTime = 0;
let timerInterval;
let recordingData = [];

const recordPauseButton = document.getElementById("recordPause");
const stopButton = document.getElementById("stop");
const updateButton = document.getElementById("update");
const timer = document.getElementById("timer");

recordPauseButton.addEventListener("click", () => {
	if (!recording) {
		startRecording();
	} else {
		paused ? resumeRecording() : pauseRecording();
	}
});

stopButton.addEventListener("click", stopRecording);

function startRecording() {
	recording = true;
	paused = false;
	startTime = Date.now() - elapsedTime;
	timerInterval = setInterval(updateTimer, 1000);
	recordPauseButton.innerText = "Pause";
}

function pauseRecording() {
	paused = true;
	clearInterval(timerInterval);
	elapsedTime = Date.now() - startTime;
	recordPauseButton.innerText = "Resume";
}

function resumeRecording() {
	paused = false;
	startTime = Date.now() - elapsedTime;
	timerInterval = setInterval(updateTimer, 1000);
	recordPauseButton.innerText = "Pause";
}

function stopRecording() {
	clearInterval(timerInterval);
	recording = false;
	paused = false;
	elapsedTime = 0;
	timer.innerText = "00:00:00";
	recordPauseButton.innerText = "Record";
	downloadRecording();
}

function updateTimer() {
	const currentTime = Date.now() - startTime;
	timer.innerText = formatTime(currentTime);
}

function formatTime(ms) {
	const totalSeconds = Math.floor(ms / 1000);
	const hours = Math.floor(totalSeconds / 3600);
	const minutes = Math.floor((totalSeconds % 3600) / 60);
	const seconds = totalSeconds % 60;
	return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
}

function saveCurrentState() {
	if (recording && !paused) {
		const currentTime = Date.now() - startTime;
		const slide = document.getElementById("goTo").value;
		const slideContent = document.getElementById("currentSlide").innerHTML;
		const slideTitle = slideContent.querySelector('h2').innerText);
		recordingData.push({ time: parseFloat((currentTime /1000).toFixed(1)), slide: slide, title: slideTitle, content: slideContent });
	}
}
window.saveCurrentState = saveCurrentState

function downloadRecording() {
	const dataStr = "data:text/json;charset=utf-8," + "slidesData=" + encodeURIComponent(JSON.stringify(recordingData));
	const downloadAnchorNode = document.createElement('a');
	downloadAnchorNode.setAttribute("href", dataStr);
	downloadAnchorNode.setAttribute("download", "recording.json");
	document.body.appendChild(downloadAnchorNode);
	downloadAnchorNode.click();
	downloadAnchorNode.remove();
	recordingData = [];
}
