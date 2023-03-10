import requestIp from 'request-ip';
import Koa from 'koa';

// koa has built-in IP, but maybe not as reliable... see https://github.com/koajs/koa/issues/599
// will check with some real data
export async function detectClientIp(ctx: Koa.Context, next: Koa.Next) {
  ctx.state.clientIp = requestIp.getClientIp(ctx.request);
  return next();
}
