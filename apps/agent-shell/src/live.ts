/**
 * Live Shell wiring against a real kernel-server (M5 Lane-TSC).
 *
 * The Shell remains a non-authority client: HttpSseTransport only moves
 * envelopes; displayed state still comes exclusively from authority
 * projections ingested over watch.
 */

import {
  HttpSseTransport,
  TaskChannelClient,
  taskCredential,
  type ClientContext,
} from "@cognitiveos/sdk-ts";

import { ShellSession } from "./session.js";

export interface LiveShellInit {
  readonly baseUrl: string;
  /** Task-channel bearer; never reuse a management credential. */
  readonly bearer: string;
  readonly context: ClientContext;
  readonly credentialId?: string;
  readonly deadline?: () => string;
}

/** Construct a ShellSession bound to the task channel over HTTP/SSE. */
export function createLiveShellSession(init: LiveShellInit): ShellSession {
  const transport = new HttpSseTransport({
    baseUrl: init.baseUrl,
    channel: "task",
    bearer: init.bearer,
  });
  const credential = taskCredential({
    credentialId: init.credentialId ?? "task-live",
    principalRef: init.context.sender,
    secret: init.bearer,
  });
  const client = new TaskChannelClient(credential, init.context, { transport });
  return new ShellSession({
    client,
    deadline: init.deadline ?? (() => new Date().toISOString()),
  });
}
