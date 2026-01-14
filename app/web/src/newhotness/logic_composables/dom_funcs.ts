export function elementIsScrolledIntoView(el?: HTMLElement, container?: HTMLElement) {
  if (!el || !document.body.contains(el)) return false;
  const rect = el.getBoundingClientRect();
  if (container) {
    // Check based on the scrolling container
    const containerRect = container.getBoundingClientRect();
    return rect.top >= containerRect.top && rect.bottom <= containerRect.bottom;
  } else {
    // Check based on the browser window
    return rect.top >= 0 && rect.bottom <= window.innerHeight;
  }
}
