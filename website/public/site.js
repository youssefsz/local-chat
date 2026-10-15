const copyButton = document.querySelector('#copy-code');
copyButton?.addEventListener('click', async () => {
  try {
    await navigator.clipboard.writeText(
      document.querySelector('#example-code').textContent,
    );
    copyButton.textContent = 'Copied';
    document.querySelector('#copy-status').textContent = 'Example code copied.';
    setTimeout(() => {
      copyButton.textContent = 'Copy code';
    }, 1500);
  } catch {
    document.querySelector('#copy-status').textContent =
      'Could not access the clipboard. Select the code to copy it manually.';
  }
});

for (const button of document.querySelectorAll('[data-appearance]')) {
  button.addEventListener('click', () => {
    const dark = button.dataset.appearance === 'dark';
    const image = document.querySelector('#app-screenshot');
    image.src = dark ? '/assets/app-dark.jpg' : '/assets/app-light.jpg';
    image.alt = `Local Chat in ${dark ? 'Dark' : 'Light'} appearance, with rooms on the left and a sample conversation and code snippet on the right`;
    document.querySelector('#screenshot-link').href = image.src;
    document.querySelector('#full-screenshot').href = image.src;
    for (const option of document.querySelectorAll('[data-appearance]'))
      option.setAttribute('aria-pressed', String(option === button));
  });
}

if (document.querySelector('#release-status, #release-notes')) {
  try {
    const response = await fetch('/api/release');
    if (!response.ok) throw new Error('Release unavailable');
    const release = await response.json();
    const status = document.querySelector('#release-status');
    if (status)
      status.textContent = `Latest stable release: ${release.version}. Free and open source.`;
    const version = document.querySelector('#release-version');
    if (version) {
      version.textContent = `Local Chat ${release.version}`;
      document.querySelector('#release-date').textContent = new Date(
        release.published,
      ).toLocaleDateString(undefined, {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
      });
      document.querySelector('#release-loading').remove();
      // GitHub Markdown is untrusted. Render the reviewed highlights as text,
      // with the full, formatted notes and contributor links on GitHub.
      const highlights =
        release.notes.split('## Highlights')[1]?.split('\n## ')[0]?.trim() ||
        'Read the full release notes on GitHub.';
      for (const block of highlights.split(/\n\s*\n/)) {
        const paragraph = document.createElement('p');
        paragraph.textContent = block;
        document.querySelector('#release-notes').append(paragraph);
      }
      document.querySelector('#release-github').href = release.url;
    }
  } catch {
    const status = document.querySelector('#release-status');
    if (status)
      status.textContent =
        'Release information is temporarily unavailable. Download links will take you to GitHub.';
  }
}
