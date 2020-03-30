// Page state container
let title = document.querySelector('title');
// Page state object
let page_state = JSON.parse(title.innerHTML);

// Update scroll position on load:
window.scroll(0, page_state.scroll_top);

// Set image sizes we have data for, store sizes for new images.
document.querySelectorAll('img').forEach(function(img) {
  const width  = page_state.image_widths[img.src];
  const height = page_state.image_heights[img.src];
  let style = "";

  if (width)  { style = `${style} width:  ${width}px;`; }
  if (height) { style = `${style} height: ${height}px;`; }

  img.style = style;

  img.onload = function() {
    // Remove the style overloads on load in case the image has changed:
    img.style = "";

    // Cache calculated sizes:
    page_state.image_heights[this.src] = this.height;
    page_state.image_widths[this.src]  = this.width;
    title.innerHTML = JSON.stringify(page_state);
  };
});

// Store scroll position on scroll:
window.addEventListener('scroll', function() {
  page_state.scroll_top = window.pageYOffset;
  title.innerHTML = JSON.stringify(page_state);
});
