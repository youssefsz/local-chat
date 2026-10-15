<script lang="ts">
  import type { UpdateSnapshot, UpdateChannel } from '../lib/desktop/api';
  let {
    state,
    error = '',
    check,
    download,
    install,
    preferences,
    later,
    openDownload,
  }: {
    state: UpdateSnapshot | null;
    error?: string;
    check: () => void;
    download: () => void;
    install: () => void;
    preferences: (automatic: boolean, channel: UpdateChannel) => void;
    later: () => void;
    openDownload: () => void;
  } = $props();
  let busy = $derived(
    state?.status === 'checking' ||
      state?.status === 'downloading' ||
      state?.status === 'installing',
  );
  let release = $derived(state && 'release' in state ? state.release : null);
  let percent = $derived(
    state?.status === 'downloading' && state.total
      ? Math.min(100, Math.floor((state.downloaded / state.total) * 100))
      : null,
  );
</script>

<div class="update-body">
  {#if state}
    <p class="update-installed">Local Chat <span>{state.version}</span></p>
    <label class="checkbox-row"
      ><input
        type="checkbox"
        checked={state.automatic}
        disabled={busy || state.status === 'ready'}
        onchange={(e) => preferences(e.currentTarget.checked, state.channel)}
      />Check for updates automatically</label
    >
    <label class="update-channel"
      >Update channel
      <select
        aria-label="Update channel"
        value={state.channel}
        disabled={busy || state.status === 'ready'}
        onchange={(e) => {
          const value = e.currentTarget.value;
          if (value === 'stable' || value === 'beta')
            preferences(state.automatic, value);
        }}
      >
        <option value="stable">Stable</option><option value="beta">Beta</option>
      </select>
    </label>
    <p class="update-explanation">
      {state.channel === 'beta'
        ? 'Beta includes preview releases and newer stable versions.'
        : 'Stable includes finished releases only.'} Checks use the internet. Chat
      works offline.
    </p>
    <div class="update-result" aria-live="polite" aria-atomic="true">
      {#if state.status === 'idle'}<p>
          Check for a newer version of Local Chat.
        </p>
      {:else if state.status === 'checking'}<p>Checking for updates…</p>
      {:else if state.status === 'current'}<p>
          You’re up to date on the {state.channel} channel.
        </p>
      {:else if state.status === 'available'}<p class="update-result-title">
          Local Chat {state.release.version} is available
        </p>
      {:else if state.status === 'downloading'}<p>
          Downloading update{percent === null ? '…' : ` · ${percent}%`}
        </p>
      {:else if state.status === 'ready'}<p class="update-result-title">
          Your update is ready
        </p>
        <p>Your draft will be saved before Local Chat restarts.</p>
      {:else if state.status === 'installing'}<p>
          Installing update and restarting…
        </p>
      {:else if state.status === 'failed'}<p role="alert">
          {state.message}
        </p>{/if}
    </div>
    {#if state.status === 'downloading'}<progress
        aria-label="Update download"
        value={percent ?? undefined}
        max="100"
      ></progress>{/if}
    {#if release?.notes}<details class="update-notes">
        <summary>What’s new in {release.version}</summary>
        <p>{release.notes}</p>
      </details>{/if}
    {#if !state.in_app}<p class="update-explanation">
        This Debian installation updates through a downloaded package. Your
        saved chats stay on this computer.
      </p>{/if}
    {#if error}<p class="error-text" role="alert">{error}</p>{/if}
    <div class="dialog-actions">
      <button
        class="button"
        type="button"
        disabled={state.status === 'installing'}
        onclick={later}>{release ? 'Later' : 'Close'}</button
      >
      {#if state.status === 'available'}<button
          class="button primary"
          type="button"
          onclick={state.in_app ? download : openDownload}
          >{state.in_app ? 'Download update' : 'Download package'}</button
        >
      {:else if state.status === 'ready'}<button
          class="button primary"
          type="button"
          onclick={install}>Restart to update</button
        >
      {:else}<button
          class="button primary"
          type="button"
          onclick={check}
          disabled={busy}
          >{state.status === 'checking'
            ? 'Checking…'
            : 'Check for updates'}</button
        >{/if}
    </div>
  {:else}<p>Loading update settings…</p>{/if}
</div>
