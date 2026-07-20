/**
 * Channel binding and isolation tests (REQ-SHELL-CHANNEL-001,
 * REQ-AKP-SHELL-001..003, REQ-AKP-MGMT-001..003; vector
 * `shell-channel-isolation-003.json`; rule 11-typescript-clients).
 *
 * One client instance binds exactly one channel; credentials and cache keys
 * never cross channels. The `@ts-expect-error` lines are compile-time
 * negatives: the build breaks if cross-channel assignment ever type-checks.
 */

import assert from "node:assert/strict";
import { test } from "node:test";

import {
  ChannelBindingViolation,
  managementCredential,
  ProjectionStore,
  taskCredential,
  type ChannelCredential,
} from "./channel.js";

const TASK_CRED = taskCredential({
  credentialId: "cred-task-1",
  principalRef: "principal://tenant-a/user-1",
  secret: "task-secret-A",
});

const MGMT_CRED = managementCredential({
  credentialId: "cred-mgmt-1",
  principalRef: "principal://tenant-a/admin-1",
  secret: "mgmt-secret-B",
});

test("credentials are channel-branded at the type level", () => {
  const task: ChannelCredential<"task"> = TASK_CRED;
  // @ts-expect-error a management credential is not assignable to a task slot
  const smuggled: ChannelCredential<"task"> = MGMT_CRED;
  // @ts-expect-error a task credential is not assignable to a management slot
  const reverse: ChannelCredential<"management"> = TASK_CRED;
  assert.equal(task.channel, "task");
  assert.equal(smuggled.channel, "management");
  assert.equal(reverse.channel, "task");
});

test("projection stores partition by channel and credential; no cross-channel hits", () => {
  const taskStore = new ProjectionStore(TASK_CRED);
  const mgmtStore = new ProjectionStore(MGMT_CRED);

  taskStore.ingest("task://tenant-a/task-1", 3, { status: "runnable" });
  assert.deepEqual(taskStore.get("task://tenant-a/task-1")?.view, { status: "runnable" });
  // Same logical key on the management store misses: isolated keyspaces.
  assert.equal(mgmtStore.get("task://tenant-a/task-1"), undefined);

  mgmtStore.ingest("task://tenant-a/task-1", 9, { status: "blocked" });
  assert.equal(taskStore.get("task://tenant-a/task-1")?.version, 3);
  assert.equal(mgmtStore.get("task://tenant-a/task-1")?.version, 9);
});

test("projection store keeps the latest authority version and never regresses", () => {
  const store = new ProjectionStore(TASK_CRED);
  store.ingest("k", 5, { status: "waiting" });
  store.ingest("k", 4, { status: "queued" });
  assert.equal(store.get("k")?.version, 5, "stale version must not overwrite a newer projection");
  store.ingest("k", 6, { status: "runnable" });
  assert.equal(store.get("k")?.version, 6);
});

test("projection store views are read-only snapshots of authority output", () => {
  const store = new ProjectionStore(TASK_CRED);
  store.ingest("k", 1, { status: "queued" });
  const entry = store.get("k");
  assert.ok(entry);
  assert.throws(() => {
    (entry.view as { status: string }).status = "completed";
  }, /read only|Cannot assign/i);
});

test("cache partition keys never embed the credential secret", () => {
  const store = new ProjectionStore(TASK_CRED);
  store.ingest("k", 1, { status: "queued" });
  for (const key of store.debugKeys()) {
    assert.ok(!key.includes("task-secret-A"), `secret leaked into cache key: ${key}`);
  }
});

test("two stores for different credentials on the same channel stay isolated", () => {
  const other = taskCredential({
    credentialId: "cred-task-2",
    principalRef: "principal://tenant-a/user-2",
    secret: "task-secret-C",
  });
  const a = new ProjectionStore(TASK_CRED);
  const b = new ProjectionStore(other);
  a.ingest("k", 1, { status: "queued" });
  assert.equal(b.get("k"), undefined);
});

test("runtime channel guard fails closed on mixed channels", () => {
  assert.throws(
    () => ProjectionStore.assertChannel(TASK_CRED, "management"),
    (error: unknown) =>
      error instanceof ChannelBindingViolation && error.code === "SHELL_CHANNEL_BINDING_MISMATCH",
  );
});
