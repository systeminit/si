import {
  ConsoleSpanExporter,
  SimpleSpanProcessor,
} from "@opentelemetry/tracing";
import { WebTracerProvider } from "@opentelemetry/web";
import { DocumentLoad } from "@opentelemetry/plugin-document-load";
import { UserInteractionPlugin } from "@opentelemetry/plugin-user-interaction";
import { ZoneContextManager } from "@opentelemetry/context-zone";
import { CollectorExporter } from "@opentelemetry/exporter-collector";
import * as api from "@opentelemetry/api";

import { auth } from "@/auth";

const provider = new WebTracerProvider({
  //plugins: [new DocumentLoad() as any, new UserInteractionPlugin()],
  plugins: [new DocumentLoad() as any],
});
provider.addSpanProcessor(
  new SimpleSpanProcessor(
    new CollectorExporter({
      serviceName: "si-web-app",
    }),
  ),
);
provider.addSpanProcessor(new SimpleSpanProcessor(new ConsoleSpanExporter()));
provider.register({
  contextManager: new ZoneContextManager(),
});

export const tracer = provider.getTracer("si-web-app");

interface PageLoadMetaData {
  timing_unload_ms: number;
  timing_dns_end_ms: number;
  timing_ssl_end_ms: number;
  timing_response_end_ms: number;
  timing_dom_interactive_ms: number;
  timing_dom_complete_ms: number;
  timing_dom_loaded_ms: number;
  timing_total_duration_ms: number;
  user_agent: string;
  window_height: number;
  window_width: number;
  screen_height: number;
  screen_width: number;
  connection_type_effective?: string;
  connection_rtt?: number;
  timing_first_paint_ms?: number;
  timing_first_contentful_paint_ms?: number;
  redirect_count?: number;
  [key: string]: any;
}

class Telemetry {
  currentRoute: api.Span | undefined;
  session: api.Span;

  constructor() {
    this.session = tracer.startSpan(
      "web.session",
      undefined,
      api.context.active(),
    );
    this.session.setAttribute("web.session", true);
  }

  routeSpan(name: string): api.Span {
    if (this.currentRoute) {
      this.currentRoute.end();
      this.currentRoute = undefined;
    }
    this.currentRoute = this.createSpan(name);
    return this.currentRoute;
  }

  createSpan(
    name: string,
    options?: api.SpanOptions,
    context?: api.Context,
  ): api.Span {
    let parentSpan;
    if (this.currentRoute) {
      parentSpan = this.currentRoute;
    } else {
      parentSpan = this.session;
    }
    const span = tracer.withSpan(parentSpan, _ => {
      const span = tracer.startSpan(
        name,
        options,
        context || api.context.active(),
      );
      if (auth.profile) {
        const profile = auth.profile;
        span.setAttributes({
          user_id: profile.user.id,
          billing_account_id: profile.billingAccount.id,
          organization: profile.organization.id,
          workspace: profile.workspaceDefault.id,
        });
      }
      return span;
    });
    return span;
  }

  pageLoadMetadata() {
    const nt = window.performance.timing;
    const hasPerfTimeline = !!window.performance.getEntriesByType;
    const totalDurationMS = nt.loadEventEnd - nt.connectStart;
    const metadata: PageLoadMetaData = {
      // Navigation timings, transformed from timestamps into deltas (shortened)
      timing_unload_ms: nt.unloadEventEnd - nt.navigationStart,
      timing_dns_end_ms: nt.domainLookupEnd - nt.navigationStart,
      timing_ssl_end_ms: nt.connectEnd - nt.navigationStart,
      timing_response_end_ms: nt.responseEnd - nt.navigationStart,
      timing_dom_interactive_ms: nt.domInteractive - nt.navigationStart,
      timing_dom_complete_ms: nt.domComplete - nt.navigationStart,
      timing_dom_loaded_ms: nt.loadEventEnd - nt.navigationStart,

      // Entire page load duration
      timing_total_duration_ms: totalDurationMS,

      // Client properties
      user_agent: window.navigator.userAgent,
      window_height: window.innerHeight,
      window_width: window.innerWidth,
      screen_height: window.screen && window.screen.height,
      screen_width: window.screen && window.screen.width,
    };

    // @ts-ignore - it does, on chrome.
    if (navigator.connection) {
      // Chrome-only (for now) information on internet connection type (4g, wifi, etc.)
      // https://developers.google.com/web/updates/2017/10/nic62
      metadata.connection_type_effective =
        // @ts-ignore
        navigator.connection && navigator.connection.effectiveType;
      // @ts-ignore
      metadata.connect_rtt =
        // @ts-ignore
        navigator.connection && navigator.connection.rtt;
    }

    // PerformancePaintTiming data (Chrome only for now)
    if (hasPerfTimeline) {
      let paints = window.performance.getEntriesByType("paint");

      // Loop through array of two PerformancePaintTimings and send both
      for (const paint of paints) {
        if (paint.name === "first-paint") {
          metadata.timing_first_paint_ms = paint.startTime;
        } else if (paint.name === "first-contentful-paint") {
          metadata.timing_first_contentful_paint_ms = paint.startTime;
        }
      }
    }

    // Redirect Count (inconsistent browser support)
    metadata.redirect_count =
      window.performance.navigation &&
      window.performance.navigation.redirectCount;

    return metadata;
  }

  end() {
    if (this.currentRoute) {
      this.currentRoute.end();
    }
    if (this.session.isRecording()) {
      const profile = auth.profile;
      if (profile) {
        this.session.setAttributes({
          user_id: profile.user.id,
          billing_account_id: profile.billingAccount.id,
          organization: profile.organization.id,
          workspace_default: profile.workspaceDefault.id,
          ...this.pageLoadMetadata(),
        });
      }
    } else {
      this.session.setAttributes(this.pageLoadMetadata());
    }
    this.session.end();
  }
}

export const telemetry = new Telemetry();
