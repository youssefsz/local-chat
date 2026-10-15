import { test } from 'node:test';
import assert from 'node:assert/strict';
import worker, { packages, parseRelease, latestRelease } from '../worker.js';

const release = () => ({
  tag_name: 'v0.0.1',
  draft: false,
  prerelease: false,
  published_at: '2026-10-15T12:00:00Z',
  body: '<!-- internal marker -->\n## Highlights\nLocal conversations.',
  assets: Object.values(packages).map((suffix) => ({
    name: `Local-Chat_0.0.1_${suffix}`,
    state: 'uploaded',
    browser_download_url: `https://github.com/youssefsz/local-chat/releases/download/v0.0.1/Local-Chat_0.0.1_${suffix}`,
  })),
});
test('only complete published stable releases supply download URLs', () => {
  assert.equal(parseRelease(release()).version, '0.0.1');
  assert.equal(
    parseRelease(release()).notes,
    '## Highlights\nLocal conversations.',
  );
  for (const value of [
    { ...release(), draft: true },
    { ...release(), prerelease: true },
    { ...release(), tag_name: 'v0.1.0-beta.6' },
    { ...release(), assets: [] },
  ])
    assert.throws(() => parseRelease(value));
  const tampered = release();
  tampered.assets[0].browser_download_url = 'https://example.invalid/installer';
  assert.throws(() => parseRelease(tampered));
});
test('release metadata is cached and upstream failure stays an error', async () => {
  let stored;
  const cache = {
    match: async () => stored?.clone(),
    put: async (_, response) => {
      stored = response;
    },
  };
  assert.equal(
    (await latestRelease(async () => Response.json(release()), cache)).version,
    '0.0.1',
  );
  assert.equal(
    (
      await latestRelease(() => {
        throw new Error('cache should serve');
      }, cache)
    ).version,
    '0.0.1',
  );
  await assert.rejects(
    latestRelease(async () => new Response('', { status: 403 }), null),
  );
});
test('unknown platforms and write requests cannot trigger an upstream request', async () => {
  assert.equal(
    (
      await worker.fetch(
        new Request('https://local-chat.youssef.tn/download/arm64'),
        {},
      )
    ).status,
    404,
  );
  assert.equal(
    (
      await worker.fetch(
        new Request('https://local-chat.youssef.tn/api/release', {
          method: 'POST',
        }),
        {},
      )
    ).status,
    405,
  );
});
test('downloads use temporary redirects, metadata failure has an honest fallback', async () => {
  const original = globalThis.fetch;
  try {
    globalThis.fetch = async () => Response.json(release());
    const redirect = await worker.fetch(
      new Request('https://local-chat.youssef.tn/download/macos'),
      {},
    );
    assert.equal(redirect.status, 302);
    assert.equal(
      redirect.headers.get('location'),
      parseRelease(release()).downloads.macos,
    );
    assert.equal(redirect.headers.get('cache-control'), 'no-store');
    globalThis.fetch = async () => {
      throw new Error('Offline');
    };
    assert.equal(
      (
        await worker.fetch(
          new Request('https://local-chat.youssef.tn/api/release'),
          {},
        )
      ).status,
      503,
    );
    assert.equal(
      (
        await worker.fetch(
          new Request('https://local-chat.youssef.tn/download/windows'),
          {},
        )
      ).headers.get('location'),
      'https://github.com/youssefsz/local-chat/releases/latest',
    );
  } finally {
    globalThis.fetch = original;
  }
});
