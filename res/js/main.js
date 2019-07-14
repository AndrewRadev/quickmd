// Update scroll position on load:
document.addEventListener('readystatechange', function() {
  if (document.readyState == 'interactive') {
    const title = document.querySelector('title');
    window.scroll(0, title.innerHTML);
  }
});

// Store scroll position on scroll:
window.addEventListener('scroll', function() {
  let title = document.querySelector('title');
  title.innerHTML = window.pageYOffset.toString();
});
