import mitt from "mitt";
import { ref } from "vue";

export const SHOW_CACHED_APP_NOTIFICATION_EVENT = "showCachedAppNotification";

export const cachedAppEmitter = mitt();

export const cachedAppNotificationIsOpen = ref(false);
