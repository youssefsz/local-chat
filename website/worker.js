// Public GitHub release metadata only. No access to app conversations or accounts.
const REPO = 'youssefsz/local-chat';
const PREFIX = `https://github.com/${REPO}/releases/download/`;
export const packages = {
  macos: 'macos-arm64.dmg',
  windows: 'windows-x64.exe',
  linux: 'linux-x64.AppImage',
  debian: 'linux-x64.deb',
};

export function parseRelease(data) {
  if (
    !data ||
    data.draft ||
    data.prerelease ||
    !/^v\d+\.\d+\.\d+$/.test(data.tag_name) ||
    !Array.isArray(data.assets)
  ) {
    throw new Error('No stable release');
  }
  const version = data.tag_name.slice(1);
  const downloads = {};
  for (const [platform, suffix] of Object.entries(packages)) {
    const name = `Local-Chat_${version}_${suffix}`;
    const asset = data.assets.find(
      (a) => a.name === name && a.state === 'uploaded',
    );
    const expected = `${PREFIX}${data.tag_name}/${name}`;
    if (!asset || asset.browser_download_url !== expected)
      throw new Error('Incomplete release');
    downloads[platform] = expected;
  }
  return {
    version,
    downloads,
    url: `https://github.com/${REPO}/releases/tag/${data.tag_name}`,
    notes:
      typeof data.body === 'string'
        ? data.body
            .slice(0, 30000)
            .replace(/<!--[\s\S]*?-->/g, '')
            .trim()
        : '',
    published: data.published_at,
    checksums: `${PREFIX}${data.tag_name}/SHA256SUMS`,
  };
}

export async function latestRelease(
  fetcher = fetch,
  cache = globalThis.caches?.default,
) {
  const key = new Request('https://local-chat.youssef.tn/api/release');
  const cached = await cache?.match(key);
  if (cached) return cached.json();
  const response = await fetcher(
    `https://api.github.com/repos/${REPO}/releases/latest`,
    {
      headers: {
        Accept: 'application/vnd.github+json',
        'User-Agent': 'Local-Chat-Website',
      },
      signal: AbortSignal.timeout(8000),
      cf: { cacheTtl: 300, cacheEverything: true },
    },
  );
  if (!response.ok) throw new Error('Release service unavailable');
  const release = parseRelease(await response.json());
  await cache?.put(
    key,
    Response.json(release, {
      headers: { 'Cache-Control': 'public, max-age=300' },
    }),
  );
  return release;
}

export default {
  async fetch(request, env) {
    const { pathname } = new URL(request.url);
    if (request.method !== 'GET' && request.method !== 'HEAD') {
      return new Response('Method not allowed', {
        status: 405,
        headers: { Allow: 'GET, HEAD' },
      });
    }
    if (pathname === '/api/release' || pathname.startsWith('/download/')) {
      const platform = pathname.slice('/download/'.length);
      if (pathname !== '/api/release' && !Object.hasOwn(packages, platform))
        return new Response('Not found', { status: 404 });
      try {
        const release = await latestRelease();
        if (pathname === '/api/release') {
          return Response.json(release, {
            headers: {
              'Cache-Control': 'public, max-age=300',
              'X-Content-Type-Options': 'nosniff',
            },
          });
        }
        return new Response(null, {
          status: 302,
          headers: {
            Location: release.downloads[platform],
            'Cache-Control': 'no-store',
          },
        });
      } catch {
        if (pathname === '/api/release')
          return Response.json(
            { error: 'Release information is temporarily unavailable.' },
            { status: 503, headers: { 'Cache-Control': 'no-store' } },
          );
        // A working release page is a better recovery than a dead download button.
        return new Response(null, {
          status: 302,
          headers: {
            Location: `https://github.com/${REPO}/releases/latest`,
            'Cache-Control': 'no-store',
          },
        });
      }
    }
    return env.ASSETS.fetch(request);
  },
};
