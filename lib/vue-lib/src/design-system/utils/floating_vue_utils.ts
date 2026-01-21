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
  "apply-button" : {
    $extend: "tooltip",
  },
  "attribute-docs": {
    $extend: "instant-show",
    html: true,
    placement: "left-start",
  },
  "attribute-source-icon": {
    $extend: "tooltip",
  },
  notifications: {
    $extend: "instant-show",
  },
};
