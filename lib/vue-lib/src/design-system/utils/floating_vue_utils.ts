export const FLOATING_VUE_THEMES = {
  html: {
    $extend: "tooltip",
    html: true,
  },
  "instant-show": {
    $extend: "tooltip",
    instantMove: true,
    delay: { show: 0, hide: 100 },
  },
  "user-info": {
    $extend: "instant-show",
    html: true,
  },
  "w-380": {
    $extend: "tooltip",
  },
  "attribute-source-icon": {
    $extend: "tooltip",
  },
  notifications: {
    $extend: "instant-show",
  },
};
