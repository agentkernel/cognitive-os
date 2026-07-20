export const siteConfig = {
  fallbackOrigin: "http://localhost:3100",
  repositorySnapshot: {
    commit: "b626e88be3b985399051e6e7624223b9cb38e7c6",
    mergedAt: "2026-07-20T07:30:03+08:00",
    capturedAt: "2026-07-20T07:41:57+08:00",
  },
  contentLicense: "All rights reserved / UNLICENSED",
} as const;

export function getSiteOrigin(): URL {
  const configured = process.env.NEXT_PUBLIC_SITE_URL;
  return new URL(configured || siteConfig.fallbackOrigin);
}

export function hasPublishableOrigin(): boolean {
  const origin = getSiteOrigin();
  return (
    origin.protocol === "https:" &&
    origin.username === "" &&
    origin.password === "" &&
    origin.pathname === "/" &&
    origin.search === "" &&
    origin.hash === "" &&
    origin.hostname !== "localhost" &&
    origin.hostname !== "127.0.0.1" &&
    origin.hostname !== "::1" &&
    origin.hostname !== "example.com" &&
    origin.hostname !== "www.example.com" &&
    !origin.hostname.endsWith(".invalid")
  );
}
