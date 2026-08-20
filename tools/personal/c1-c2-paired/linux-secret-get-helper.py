#!/usr/bin/env python3
"""P9-T08 campaign helper: Secret Service SearchItems / GetSecret.

Never invoke secret-tool search or secret-tool lookup. Facts go to stdout as
JSON. Secret material is written only to the caller-supplied material fd.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys

SECRET_SHAPED = re.compile(
    r"sk-[A-Za-z0-9]{10,}|BEGIN [A-Z ]+PRIVATE KEY|-----BEGIN"
)


def fail(reason: str, code: int = 2) -> None:
    json.dump({"ok": False, "reason": reason}, sys.stdout, separators=(",", ":"))
    sys.stdout.write("\n")
    raise SystemExit(code)


def main() -> None:
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("--material-fd", type=int, default=3)
    parser.add_argument("--paths-only", action="store_true")
    args, unknown = parser.parse_known_args()
    if unknown:
        fail("unknown helper argument refused")

    raw = sys.stdin.read()
    try:
        attributes = json.loads(raw)
    except json.JSONDecodeError:
        fail("attributes json is invalid")
    if not isinstance(attributes, dict) or not attributes:
        fail("attributes object is required")
    for key, value in attributes.items():
        if not isinstance(key, str) or not isinstance(value, str):
            fail("attribute keys and values must be strings")
        if SECRET_SHAPED.search(key) or SECRET_SHAPED.search(value):
            fail("secret-shaped attribute refused")

    try:
        import dbus
    except ImportError:
        fail("python dbus module is not available")

    try:
        bus = dbus.SessionBus()
        service = bus.get_object("org.freedesktop.secrets", "/org/freedesktop/secrets")
        iface = dbus.Interface(service, "org.freedesktop.Secret.Service")
        _output, session_path = iface.OpenSession(
            "plain", dbus.String("", variant_level=1)
        )
        attr_map = dbus.Dictionary(attributes, signature="ss")
        unlocked, locked = iface.SearchItems(attr_map)
    except Exception:
        fail("secret service search failed")

    def last_component(path: str) -> str:
        return str(path).rstrip("/").rsplit("/", 1)[-1]

    facts = {
        "ok": True,
        "item_count_unlocked": int(len(unlocked)),
        "item_count_locked": int(len(locked)),
        "item_suffixes": [last_component(str(path)) for path in list(unlocked) + list(locked)],
        "paths_only": bool(args.paths_only),
        "material_written": False,
        "material_bytes": 0,
    }

    if args.paths_only:
        json.dump(facts, sys.stdout, separators=(",", ":"))
        sys.stdout.write("\n")
        return

    if len(locked) and not len(unlocked):
        fail("secret item is locked and prompting is refused")
    if len(unlocked) == 0:
        fail("secret item not found")
    if len(unlocked) != 1:
        fail("secret item search was not unique")

    try:
        item = bus.get_object("org.freedesktop.secrets", str(unlocked[0]))
        item_iface = dbus.Interface(item, "org.freedesktop.Secret.Item")
        secret = item_iface.GetSecret(session_path)
        material = bytes(secret[2])
    except Exception:
        fail("secret service get failed")

    if not material:
        fail("secret material was empty")

    try:
        sink = os.fdopen(args.material_fd, "wb", closefd=False)
        sink.write(material)
        sink.flush()
    except Exception:
        fail("material fd write failed")

    facts["material_written"] = True
    facts["material_bytes"] = len(material)
    facts["item_suffixes"] = [last_component(str(unlocked[0]))]
    json.dump(facts, sys.stdout, separators=(",", ":"))
    sys.stdout.write("\n")


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        raise
    except Exception:
        fail("helper failed closed")
