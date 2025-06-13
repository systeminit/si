export function elementIsScrolledIntoView(el?: HTMLElement) {
  if (!el) return false;
  const rect = el.getBoundingClientRect();
  return rect.top >= 0 && rect.bottom <= window.innerHeight;
}
