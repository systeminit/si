#include <stdio.h>
#include <stdarg.h>

/*
 * Helper function to expose vsnprintf for logging purposes. va_list is a nightmare to
 * handle directly from rust it seems.
 */
int wrapped_vsnprintf(char * buf, size_t size, const char *format, va_list ap) {
  return vsnprintf(buf, size, format, ap);
}