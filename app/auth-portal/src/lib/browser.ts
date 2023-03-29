const DETECT_MOBILE_BROWSER_REGEX =
  /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i;

export const BROWSER_IS_MOBILE =
  !import.meta.env.SSR && DETECT_MOBILE_BROWSER_REGEX.test(navigator.userAgent);
