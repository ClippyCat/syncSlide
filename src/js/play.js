const video = document.getElementById('myVideo');
const slidesContainer = document.getElementById('currentSlide');
const slidesData = [{"time":1,"slide":"0","content":"<h1>test slides</h1><h2>Text</h2><p>Hello world</p>"},{"time":13,"slide":"1","content":"<h1>test slides</h1><h2>List</h2><ul>\n<li>1</li>\n<li>2</li>\n<li>3</li>\n</ul>"},{"time":19,"slide":"2","content":"<h1>test slides</h1><h2>Video</h2><iframe width=\"560\" height=\"315\" src=\"https://www.youtube.com/embed/ehTIhQpj9ys\" allow=\"autoplay; encrypted-media; gyroscope; picture-in-picture\" allowfullscreen=\"\">\n</iframe>"}];

video.ontimeupdate = (event) => {
  const currentTime = Math.floor(video.currentTime);
  slidesData.forEach(slideData => {
    if (slideData.time == currentTime) {
      slidesContainer.innerHTML = slideData.content;
    }
  });
};