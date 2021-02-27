// Page state container
let title = document.querySelector('title');
// Page state object
let page_state = JSON.parse(title.innerHTML);

// Update scroll position on load:
window.scroll(0, page_state.scroll_top);

// Store scroll position on scroll:
window.addEventListener('scroll', function() {
  page_state.scroll_top = window.pageYOffset;
  title.innerHTML = JSON.stringify(page_state);
});

// Set image sizes we have data for, store sizes for new images:
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

// Create anchors for all the headings:
document.querySelectorAll('h1, h2, h3, h4, h5, h6').forEach(function(heading) {
  if (!heading.id) {
    let content = heading.innerHTML.trim().toLowerCase();
    let slug    = content.replace(/[^\w\d]+/g, '-');

    heading.id = slug;
  }
});

// Show link preview at the bottom:
let linkPreview = document.querySelector('#link-preview');
let rootUrl = window.location.href;

document.querySelectorAll('a').forEach(function(link) {
  let url = link.href;
  let description;

  if (url.startsWith(rootUrl)) {
    // it's a local anchor, let's just take that part
    description = `<strong>Jump</strong>: ${url.replace(rootUrl, '')}`
  } else {
    // it's an external URL, copy it
    description = `<strong>Copy</strong>: ${url}`

    link.addEventListener('click', function(e) {
      e.preventDefault();

      // Create a temporary text input whose contents we can "select" and copy using `execCommand`.
      let tempInput = document.createElement('input');
      tempInput.setAttribute('type', 'text');
      tempInput.setAttribute('value', url);
      document.body.insertBefore(tempInput, linkPreview);

      tempInput.select();
      document.execCommand('copy');
      document.body.removeChild(tempInput);

      linkPreview.innerHTML = "<strong>Copied to clipboard!</strong>";
    });
  }

  link.addEventListener('mouseenter', function() {
    linkPreview.innerHTML = description;
    linkPreview.classList.remove('hidden');
    linkPreview.classList.add('visible');
  });

  link.addEventListener('mouseleave', function() {
    linkPreview.classList.remove('visible');
    linkPreview.classList.add('hidden');
  });
});
