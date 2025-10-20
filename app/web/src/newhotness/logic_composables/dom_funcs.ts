export function elementIsScrolledIntoView(el?: HTMLElement) {
  if (!el || !document.body.contains(el)) return false;
  const rect = el.getBoundingClientRect();
  return rect.top >= 0 && rect.bottom <= window.innerHeight;
}
