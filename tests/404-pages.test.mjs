import assert from "node:assert/strict";
import { access, readFile } from "node:fs/promises";
import { test } from "node:test";

const siteRoot = new URL("../", import.meta.url);

test("the official fallback is the detailed terrain concept", async () => {
  const html = await readFile(new URL("404.html", siteRoot), "utf8");

  assert.match(html, /<h1 id="error-title">Off the <span>map\.<\/span><\/h1>/);
  assert.match(html, /viewBox="0 0 1600 1000"/);
  assert.match(html, /preserveAspectRatio="xMidYMid slice"/);
  assert.ok((html.match(/<path/g) ?? []).length >= 12, "the map needs enough contours to read as terrain");
  assert.equal((html.match(/<circle/g) ?? []).length, 2, "both peaks need a summit fix");
});

test("the fallback exposes only its recovery action", async () => {
  const html = await readFile(new URL("404.html", siteRoot), "utf8");

  assert.equal((html.match(/class="error-button/g) ?? []).length, 1);
  assert.match(html, /href="\/">Recenter home<\/a>/);
  assert.doesNotMatch(html, /data-request-path|coordinate-route|View all concepts|\/404s\//);
});

test("Trunk deploys the fallback and its minimal static dependencies", async () => {
  const shell = await readFile(new URL("index.html", siteRoot), "utf8");

  for (const asset of ["404.html", "404.css", "shared/tokens.css", "shared/typography.css", "shared/header.css"]) {
    assert.ok(shell.includes(`href="${asset}"`), `${asset} is absent from the deployment contract`);
  }
  assert.doesNotMatch(shell, /copy-dir" href="404s"/);
});

test("the discarded concept gallery is absent", async () => {
  await assert.rejects(access(new URL("404s/", siteRoot)));
});
