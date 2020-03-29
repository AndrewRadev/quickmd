// Update scroll position on load:
document.addEventListener('readystatechange', function() {
  if (document.readyState == 'interactive') {
    const title = document.querySelector('title');
    const page_state = JSON.parse(title.innerHTML);
    window.scroll(0, page_state.scroll_top);
  }
});

// Store scroll position on scroll:
window.addEventListener('scroll', function() {
  let title = document.querySelector('title');
  const page_state = JSON.parse(title.innerHTML);
  page_state.scroll_top = window.pageYOffset;

  title.innerHTML = JSON.stringify(page_state);
});
