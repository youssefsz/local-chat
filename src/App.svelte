<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { invoke } from '@tauri-apps/api/core';
  import Icon from './components/Icon.svelte';
  import Symbols from './components/Symbols.svelte';
  import MessageView from './components/MessageView.svelte';
  import { api, desktop } from './lib/desktop/api';
  import type {
    Snapshot,
    Room,
    Message,
    SearchResult,
    Theme,
  } from './lib/desktop/api';
  let appState = $state<Snapshot | null>(null);
  let context = $state('');
  let active = $state('general');
  let roomList = $state<Room[]>([]);
  let messages = $state<Message[]>([]);
  let content = $state('');
  let code = $state(false);
  let error = $state('');
  let sending = $state(false);
  let people = $state(false);
  let menu = $state(false);
  let archived = $state(false);
  let previous = $state(false);
  // svelte-ignore non_reactive_update
  let conversation: HTMLElement;
  // svelte-ignore non_reactive_update
  let input: HTMLTextAreaElement;
  let searchDialog: HTMLDialogElement;
  let roomDialog: HTMLDialogElement;
  let settingsDialog: HTMLDialogElement;
  let query = $state('');
  let scope = $state<string | null>(null);
  let results = $state<SearchResult[]>([]);
  let searching = $state(false);
  let edit = $state(false);
  let savingRoom = $state(false);
  let roomName = $state('');
  let description = $state('');
  let profile = $state('');
  let theme = $state<Theme>('system');
  let notifications = $state(true);
  let copied = $state('');
  let toast = $state('');
  let newMessages = $state(false);
  let atBottom = true;
  let hasOlder = $state(false);
  let hasNewer = $state(false);
  let highlighted = $state('');
  let highlightTimer: ReturnType<typeof setTimeout>;
  let loading = $state(false);
  let selectionVersion = 0;
  let searchVersion = 0;
  let copyTimer: ReturnType<typeof setTimeout>;
  let toastTimer: ReturnType<typeof setTimeout>;
  let refreshMessages = false;
  let hiddenPending = false;
  let refreshTimer: ReturnType<typeof setTimeout>;
  let viewTimer: ReturnType<typeof setTimeout>;
  let draftQueue: Promise<void> = Promise.resolve();
  const scrolls = new Map<string, number>();
  let current = $derived(context === appState?.context);
  let room = $derived(roomList.find((r) => r.id === active));
  let connected = $derived(
    appState?.peers.filter((p) => p.status === 'connected') ?? [],
  );
  let owner = $derived(
    room?.owner === appState?.identity && active !== 'general' && current,
  );
  let hasDialog = () => !!document.querySelector('dialog[open]');
  let initials = (name: string) =>
    name
      .trim()
      .split(/\s+/)
      .slice(0, 2)
      .map((x) => x[0])
      .join('')
      .toUpperCase();
  let fail = (e: unknown) => {
    error = String(e);
  };
  function notify(text: string) {
    toast = text;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      toast = '';
    }, 2000);
  }
  async function applyTheme(value: Theme) {
    const dark =
      value === 'dark' ||
      (value === 'system' && matchMedia('(prefers-color-scheme:dark)').matches);
    document.documentElement.dataset.theme = dark ? 'dark' : 'light';
    document.documentElement.style.colorScheme = dark ? 'dark' : 'light';
    if (desktop)
      await getCurrentWindow().setTheme(value === 'system' ? null : value);
  }
  function persistDraft() {
    if (!current || !appState || !room || room.archived) return;
    const saved = { context, room: active, content, code };
    draftQueue = draftQueue
      .catch(() => {})
      .then(() =>
        api.saveDraft(saved.context, saved.room, saved.content, saved.code),
      )
      .catch(fail);
  }
  async function selectRoom(
    id: string,
    nextContext = context,
    messageId: string | null = null,
  ) {
    const started = performance.now();
    persistDraft();
    saveView();
    if (conversation) {
      scrolls.set(context + active, conversation.scrollTop);
      void api
        .local(context, active, null, null, conversation.scrollTop)
        .catch(fail);
    }
    const version = ++selectionVersion;
    loading = true;
    menu = false;
    try {
      await draftQueue;
      const list =
        nextContext === appState?.context
          ? appState.rooms
          : await api.rooms(nextContext);
      const selected = list.find((r) => r.id === id);
      if (!selected) return;
      const view = messageId ? null : await api.view(nextContext, id);
      const [page, draft, position] = await Promise.all([
        api.messages(nextContext, id, null, messageId ?? view?.anchor ?? null),
        api.draft(nextContext, id),
        api.local(nextContext, id),
      ]);
      if (version !== selectionVersion) return;
      context = nextContext;
      active = id;
      roomList = list;
      messages = page.items;
      hasOlder = page.has_older;
      hasNewer = page.has_newer;
      content = draft.content;
      code = draft.code;
      newMessages = false;
      await tick();
      if (messageId) {
        highlighted = messageId;
        clearTimeout(highlightTimer);
        highlightTimer = setTimeout(() => (highlighted = ''), 2000);
        document
          .getElementById('msg-' + messageId)
          ?.scrollIntoView({ block: 'center' });
        document
          .getElementById('msg-' + messageId)
          ?.focus({ preventScroll: true });
      } else if (view) {
        const anchor = document.getElementById('msg-' + view.anchor);
        if (anchor)
          conversation.scrollTop +=
            anchor.getBoundingClientRect().top -
            conversation.getBoundingClientRect().top -
            view.offset;
      } else
        conversation.scrollTop =
          scrolls.get(context + active) ??
          (position >= 0 ? position : conversation.scrollHeight);
      atBottom =
        conversation.scrollHeight -
          conversation.scrollTop -
          conversation.clientHeight <
        50;
      if (atBottom) await markRead();
      void api.recordRoomSwitch(performance.now() - started).catch(fail);
    } catch (e) {
      fail(e);
    } finally {
      if (version === selectionVersion) loading = false;
    }
  }
  async function markRead() {
    const last = messages.at(-1);
    if (last && current) {
      await api.local(context, active, null, last.logical);
      roomList = roomList.map((r) =>
        r.id === active ? { ...r, unread: 0 } : r,
      );
    }
  }
  async function refresh(readMessages = true) {
    try {
      const snapshot = await api.snapshot();
      const moved = appState && snapshot.context !== appState.context;
      const wasCurrent = current;
      appState = snapshot;
      if (moved) {
        notify(
          'Network changed. Earlier history is kept in Previous networks.',
        );
        if (wasCurrent || !context)
          await selectRoom('general', snapshot.context);
        else {
          roomList = await api.rooms(context);
        }
        return;
      }
      if (!context && snapshot.context) {
        await selectRoom('general', snapshot.context);
        return;
      }
      if (current) roomList = snapshot.rooms;
      if (!room || loading || !current || !readMessages) return;
      const version = selectionVersion;
      const page = await api.messages(context, active);
      if (version !== selectionVersion) return;
      if (atBottom) {
        const existing = new Map(messages.map((m) => [m.id, m]));
        messages = page.items.map((m) => {
          const old = existing.get(m.id);
          return old && old.delivery === m.delivery ? old : m;
        });
        hasOlder = page.has_older;
        hasNewer = page.has_newer;
        await tick();
        conversation.scrollTop = conversation.scrollHeight;
        await markRead();
      } else {
        const latest = page.items.at(-1);
        if (latest && latest.id !== messages.at(-1)?.id) newMessages = true;
        const updates = new Map(page.items.map((m) => [m.id, m]));
        messages = messages.map((m) => updates.get(m.id) ?? m);
      }
    } catch (e) {
      fail(e);
    }
  }
  async function older() {
    const first = messages[0];
    if (!first || loading) return;
    loading = true;
    const version = selectionVersion;
    const anchor = Array.from(
      conversation.querySelectorAll<HTMLElement>('[data-message]'),
    ).find(
      (el) =>
        el.getBoundingClientRect().bottom >
        conversation.getBoundingClientRect().top,
    );
    const anchorId = anchor?.id;
    const anchorTop = anchor?.getBoundingClientRect().top ?? 0;
    try {
      const page = await api.messages(context, active, first.id);
      if (version !== selectionVersion) return;
      messages = [...page.items, ...messages].slice(0, 160);
      hasOlder = page.has_older;
      hasNewer = page.has_newer;
      await tick();
      const preserved = anchorId ? document.getElementById(anchorId) : null;
      if (preserved)
        conversation.scrollTop +=
          preserved.getBoundingClientRect().top - anchorTop;
    } catch (e) {
      fail(e);
    } finally {
      loading = false;
    }
  }
  async function bottom() {
    const page = await api.messages(context, active);
    messages = page.items;
    hasOlder = page.has_older;
    hasNewer = page.has_newer;
    await tick();
    conversation.scrollTop = conversation.scrollHeight;
    atBottom = true;
    newMessages = false;
    await markRead();
  }
  function saveView() {
    if (!conversation || loading) return;
    const top = conversation.getBoundingClientRect().top;
    const anchor = Array.from(
      conversation.querySelectorAll<HTMLElement>('[data-message]'),
    ).find((el) => el.getBoundingClientRect().bottom > top);
    const id = anchor?.dataset.message;
    if (anchor && id)
      void api
        .saveView(context, active, id, anchor.getBoundingClientRect().top - top)
        .catch(fail);
  }
  function onScroll() {
    clearTimeout(viewTimer);
    viewTimer = setTimeout(saveView, 200);
    atBottom =
      conversation.scrollHeight -
        conversation.scrollTop -
        conversation.clientHeight <
      50;
    if (atBottom) {
      newMessages = false;
      void markRead().catch(fail);
    }
  }
  async function send() {
    if (sending || !content.trim() || !current || room?.archived) return;
    sending = true;
    const text = content;
    const target = active;
    const targetContext = context;
    try {
      await draftQueue;
      await api.send(targetContext, target, text, code);
      if (active === target && context === targetContext) {
        content = '';
        code = false;
        await bottom();
        input.focus();
      }
    } catch (e) {
      fail(e);
    } finally {
      sending = false;
    }
  }
  function composerKey(e: KeyboardEvent) {
    if (e.isComposing || e.keyCode === 229) return;
    if (e.key === 'Enter' && (code ? e.metaKey || e.ctrlKey : !e.shiftKey)) {
      e.preventDefault();
      void send();
    }
  }
  async function copy(message: Message) {
    try {
      await invoke('plugin:clipboard-manager|write_text', {
        text: message.content,
      });
      copied = message.id;
      clearTimeout(copyTimer);
      copyTimer = setTimeout(() => (copied = ''), 1500);
      notify(
        message.mode === 'code'
          ? 'Code copied. Formatting preserved.'
          : 'Message copied',
      );
    } catch (e) {
      fail(e);
    }
  }
  async function openLink(url: string) {
    try {
      if (!['http:', 'https:'].includes(new URL(url).protocol)) return;
      await invoke('plugin:opener|open_url', { url });
    } catch (e) {
      fail(e);
    }
  }
  function openSearch(roomScope: string | null = null) {
    scope = roomScope;
    query = '';
    results = [];
    searchDialog.showModal();
    searchDialog.querySelector('input')?.focus();
  }
  async function runSearch() {
    const version = ++searchVersion;
    searching = true;
    try {
      const matches = await api.search(query, scope, context);
      if (version === searchVersion) results = matches;
    } catch (e) {
      fail(e);
    } finally {
      if (version === searchVersion) searching = false;
    }
  }
  async function choose(result: SearchResult) {
    searchDialog.close();
    await selectRoom(result.message.room, result.context, result.message.id);
  }
  function openRoom(rename = false) {
    menu = false;
    error = '';
    edit = rename;
    roomName = rename ? (room?.name ?? '') : '';
    description = rename ? (room?.description ?? '') : '';
    roomDialog.showModal();
    roomDialog.querySelector('input')?.focus();
  }
  async function saveRoom(e: SubmitEvent) {
    e.preventDefault();
    if (savingRoom) return;
    savingRoom = true;
    try {
      const id = await api.manage(
        context,
        edit ? 'rename' : 'create',
        edit ? active : null,
        roomName.trim(),
        description.trim(),
      );
      appState = await api.snapshot();
      roomList = appState.rooms;
      roomDialog.close();
      await selectRoom(id);
      notify(edit ? 'Room updated' : 'Room created');
    } catch (e) {
      fail(e);
    } finally {
      savingRoom = false;
    }
  }
  async function archiveRoom() {
    if (!room) return;
    try {
      await api.manage(context, room.archived ? 'restore' : 'archive', active);
      menu = false;
      archived = true;
      await refresh();
    } catch (e) {
      fail(e);
    }
  }
  async function mute() {
    if (!room) return;
    try {
      await api.local(context, active, !room.muted);
      menu = false;
      await refresh();
    } catch (e) {
      fail(e);
    }
  }
  function settings() {
    if (!appState) return;
    error = '';
    profile = appState.profile;
    theme = appState.theme;
    notifications = appState.notifications;
    settingsDialog.showModal();
  }
  async function saveSettings(e: SubmitEvent) {
    e.preventDefault();
    try {
      await api.preferences(profile, theme, notifications);
      if (appState) appState = { ...appState, theme, profile, notifications };
      await applyTheme(theme);
      settingsDialog.close();
      await refresh();
      notify('Preferences saved');
    } catch (e) {
      fail(e);
    }
  }
  function insertLink() {
    const start = input.selectionStart;
    content =
      content.slice(0, start) + 'https://' + content.slice(input.selectionEnd);
    persistDraft();
    void tick().then(() => {
      input.focus();
      input.setSelectionRange(start, start + 8);
    });
  }
  function searchKeys(e: KeyboardEvent) {
    if (e.key === 'Enter' && e.target instanceof HTMLInputElement) {
      e.preventDefault();
      searchDialog.querySelector<HTMLButtonElement>('.search-result')?.click();
      return;
    }
    if (e.key !== 'ArrowDown' && e.key !== 'ArrowUp') return;
    const options = Array.from(
      searchDialog.querySelectorAll<HTMLButtonElement>('.search-result'),
    );
    const index = options.findIndex(
      (button) => button === document.activeElement,
    );
    e.preventDefault();
    if (e.key === 'ArrowUp' && index <= 0)
      searchDialog.querySelector('input')?.focus();
    else
      options[
        (index + (e.key === 'ArrowDown' ? 1 : -1) + options.length) %
          options.length
      ]?.focus();
  }
  function dismissMenu(e: MouseEvent) {
    if (
      menu &&
      e.target instanceof Element &&
      !e.target.closest('.room-menu-wrapper')
    )
      menu = false;
  }
  function keys(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      if (searchDialog.open) searchDialog.close();
      else if (!hasDialog()) openSearch();
    }
    if (e.key === 'Escape') {
      if (menu) document.getElementById('room-options')?.focus();
      else if (people) document.getElementById('people-trigger')?.focus();
      menu = false;
      people = false;
    }
    if (e.altKey && /^[1-9]$/.test(e.key) && !hasDialog()) {
      const selected = roomList.filter((r) => !r.archived)[Number(e.key) - 1];
      if (selected) {
        e.preventDefault();
        void selectRoom(selected.id);
      }
    }
    if (menu && e.key === 'ArrowDown') {
      e.preventDefault();
      document.querySelector<HTMLButtonElement>('.popover button')?.focus();
    }
  }
  function backdrop(e: MouseEvent) {
    if (!(e.target instanceof HTMLDialogElement)) return;
    const bounds = e.target.getBoundingClientRect();
    if (
      e.clientX < bounds.left ||
      e.clientX > bounds.right ||
      e.clientY < bounds.top ||
      e.clientY > bounds.bottom
    )
      e.target.close();
  }
  onMount(() => {
    if (!desktop) {
      error =
        'Open Local Chat as a desktop application. Local network and saved history run in the desktop app.';
      return;
    }
    let clean: (() => void) | undefined;
    const media = matchMedia('(prefers-color-scheme:dark)');
    const changeTheme = () => {
      if (appState?.theme === 'system') void applyTheme('system');
    };
    media.addEventListener('change', changeTheme);
    const visibilityChanged = () => {
      if (!document.hidden && hiddenPending) {
        hiddenPending = false;
        const readMessages = refreshMessages;
        refreshMessages = false;
        void refresh(readMessages);
      }
    };
    document.addEventListener('visibilitychange', visibilityChanged);
    void (async () => {
      try {
        appState = await api.snapshot();
        await applyTheme(appState.theme);
        if (appState.context) await selectRoom('general', appState.context);
        clean = await listen<{ kind: 'state' | 'messages' }>(
          'local-chat:changed',
          (event) => {
            refreshMessages ||= event.payload.kind === 'messages';
            if (document.hidden) {
              hiddenPending = true;
              return;
            }
            clearTimeout(refreshTimer);
            refreshTimer = setTimeout(() => {
              const readMessages = refreshMessages;
              refreshMessages = false;
              void refresh(readMessages);
            }, 60);
          },
        );
        await refresh();
        await api.ready();
      } catch (e) {
        fail(e);
        await api.ready();
      }
    })();
    return () => {
      clean?.();
      media.removeEventListener('change', changeTheme);
      document.removeEventListener('visibilitychange', visibilityChanged);
      clearTimeout(copyTimer);
      clearTimeout(toastTimer);
      clearTimeout(refreshTimer);
      clearTimeout(viewTimer);
      clearTimeout(highlightTimer);
    };
  });
</script>

<svelte:window onkeydown={keys} onclick={dismissMenu} />
<Symbols />
<div class="app">
  {#if error}<div class="error-banner" role="alert">
      <span>{error}</span><button
        class="icon-button"
        aria-label="Dismiss error"
        onclick={() => (error = '')}><Icon name="x" /></button
      >
    </div>{/if}
  {#if appState}
    <div class="shell">
      <aside class="sidebar" aria-label="Rooms and people">
        <div class="brand">
          <span class="brand-mark"><Icon name="message" /></span>Local Chat
        </div>
        <button class="search-trigger" onclick={() => openSearch()}
          ><Icon name="search" /><span class="search-label-full"
            >Find a message or room</span
          ><span class="search-label-short">Find messages</span><kbd
            >{navigator.platform.includes('Mac') ? '⌘ K' : 'Ctrl K'}</kbd
          ></button
        >
        <div class="sidebar-scroll">
          <section class="nav-section">
            <div class="section-label">
              <span>{current ? 'Shared rooms' : 'Previous network rooms'}</span
              >{#if current}<button
                  class="icon-button"
                  aria-label="Create a room"
                  title="Create a room"
                  onclick={() => openRoom()}><Icon name="plus" small /></button
                >{/if}
            </div>
            <nav class="room-list" aria-label="Shared rooms">
              {#each roomList.filter((r) => !r.archived) as r (r.id)}<button
                  class="room-button"
                  class:active={r.id === active}
                  aria-current={r.id === active ? 'page' : undefined}
                  onclick={() => selectRoom(r.id)}
                  ><Icon name="hash" /><span class="room-name">{r.name}</span
                  >{#if r.muted}<Icon
                      name="bell-off"
                      small
                    />{:else if r.unread}<span class="unread-count"
                      >{r.unread}</span
                    >{/if}{#if roomList.filter((other) => other.name === r.name).length > 1}<span
                      class="duplicate-id"
                      title={r.id}>{r.id.slice(-4)}</span
                    >{/if}</button
                >{/each}
            </nav>
            {#if roomList.some((r) => r.archived)}<button
                class="archived-toggle"
                aria-expanded={archived}
                onclick={() => (archived = !archived)}
                ><Icon name="archive" />Archived rooms</button
              >{#if archived}<nav
                  class="room-list archived-list"
                  aria-label="Archived rooms"
                >
                  {#each roomList.filter((r) => r.archived) as r (r.id)}<button
                      class="room-button"
                      class:active={r.id === active}
                      onclick={() => selectRoom(r.id)}
                      ><Icon name="archive" /><span class="room-name"
                        >{r.name}</span
                      ></button
                    >{/each}
                </nav>{/if}{/if}
          </section>
          {#if appState.contexts.some((c) => !c.current)}<button
              class="history-toggle"
              aria-expanded={previous}
              onclick={() => (previous = !previous)}
              ><Icon name="archive" small />Previous networks</button
            >{#if previous}{#each appState.contexts as c}<button
                  class="context-row"
                  class:active={context === c.id}
                  onclick={() => selectRoom('general', c.id)}
                  >{c.label}{c.current
                    ? ' · current'
                    : ' · ' + c.id.slice(0, 6)}</button
                >{/each}{/if}{/if}
          {#if !current}<button
              class="context-row"
              onclick={() => selectRoom('general', appState?.context)}
              >Return to current network</button
            >{/if}
          <section class="people-section">
            <div class="section-label">
              <span>People nearby</span><span class="online-label"
                ><i class="status-dot"></i>{connected.length}<span
                  class="connected-word">connected</span
                ></span
              >
            </div>
            {#each connected.slice(0, 5) as peer (peer.id)}<button
                class="person"
                onclick={() => (people = true)}
                ><span class="avatar"
                  >{initials(peer.name)}<i class="status-dot"></i></span
                ><span class="person-name">{peer.name}</span><span
                  class="device-label">{peer.platform}</span
                ></button
              >{/each}{#if !connected.length}<p class="status-line">
                {appState.network_status === 'unavailable'
                  ? 'Local network unavailable.'
                  : 'Looking for nearby computers…'}<br />Open Local Chat on
                another computer on this network.
              </p>{/if}
          </section>
        </div>
        <div class="sidebar-footer">
          <button class="network-card" onclick={() => (people = true)}
            ><Icon name="wifi" /><span
              ><span class="network-title"
                >{appState.network_status === 'connected'
                  ? 'Local network'
                  : appState.network_status === 'unavailable'
                    ? 'Connection unavailable'
                    : 'Discovering nearby'}</span
              ><span class="network-caption" style="display:block"
                >{connected.length
                  ? 'A shared space, right here.'
                  : 'Messages stay saved here.'}</span
              ></span
            ><Icon name="chevron" /></button
          >
          <div class="profile-row">
            <span class="avatar you">{initials(appState.profile)}</span><span
              class="profile-text"
              >{appState.profile}<small>This computer</small></span
            ><button
              class="icon-button"
              aria-label="Settings"
              title="Settings"
              onclick={settings}><Icon name="settings" /></button
            >
          </div>
        </div>
      </aside>
      <main class="main">
        <header class="room-header">
          <div class="room-heading">
            <div class="room-symbol"><Icon name="hash" /></div>
            <div class="main-header-text">
              <h1>{room?.name ?? 'General'}</h1>
              <div class="room-subtitle">
                {room?.description ||
                  'A shared room for everyone on this network.'}
              </div>
            </div>
          </div>
          <div class="header-actions">
            <button
              class="members-button"
              id="people-trigger"
              aria-label="Show nearby people"
              aria-expanded={people}
              onclick={() => (people = !people)}
              ><span class="avatar-stack"
                >{#each connected.slice(0, 3) as peer}<span class="avatar"
                    >{initials(peer.name)}</span
                  >{/each}</span
              ><Icon name="users" /><span class="member-count"
                >{connected.length} nearby</span
              ></button
            ><span class="header-divider"></span><button
              class="icon-button"
              aria-label="Search this room"
              title="Search this room"
              onclick={() => openSearch(active)}><Icon name="search" /></button
            >
            <div class="room-menu-wrapper">
              <button
                class="icon-button"
                id="room-options"
                aria-label="Room options"
                aria-expanded={menu}
                aria-controls="room-menu"
                onclick={() => (menu = !menu)}><Icon name="more" /></button
              >{#if menu}<div class="popover" id="room-menu">
                  <button onclick={mute}
                    ><Icon
                      name={room?.muted ? 'bell' : 'bell-off'}
                    />{room?.muted ? 'Unmute room' : 'Mute room'}</button
                  >{#if owner}<button onclick={() => openRoom(true)}
                      ><Icon name="edit" />Rename room</button
                    ><button onclick={archiveRoom}
                      ><Icon name="archive" />{room?.archived
                        ? 'Restore room'
                        : 'Archive room'}</button
                    >{:else}<div class="menu-note">
                      {active === 'general'
                        ? 'General is a shared system room.'
                        : current
                          ? 'Only the creator installation can manage this room.'
                          : 'Previous network history is read-only.'}
                    </div>{/if}
                </div>{/if}
            </div>
          </div>
        </header>
        <section
          class="conversation"
          aria-label="Conversation"
          bind:this={conversation}
          onscroll={onScroll}
        >
          <div class="conversation-inner">
            {#if hasOlder}<button
                class="load-older"
                disabled={loading}
                onclick={older}
                >{loading ? 'Loading…' : 'Load earlier messages'}</button
              >{:else}<div class="room-intro">
                <div class="intro-icon"><Icon name="hash" /></div>
                <h2>
                  {active === 'general' ? 'Everyone’s here.' : room?.name}
                </h2>
                <p>
                  {active === 'general'
                    ? 'One shared space for the people around you.'
                    : room?.description ||
                      'Start a conversation. Everyone on this network can read and reply.'}<br
                  />Drop a thought, a link, or something worth keeping.
                </p>
              </div>{/if}
            {#if room?.archived}<div class="archive-banner">
                <span>This room is archived. Its history is still here.</span
                >{#if owner}<button onclick={archiveRoom}>Restore room</button
                  >{/if}
              </div>{/if}
            {#if messages.length}<div class="date-divider">
                {new Date(
                  messages[0]?.timestamp ?? Date.now(),
                ).toLocaleDateString([], {
                  weekday: 'long',
                  month: 'long',
                  day: 'numeric',
                })}
              </div>
              {#each messages as message (message.id)}<MessageView
                  {message}
                  highlighted={highlighted === message.id}
                  own={message.author === appState.identity}
                  copied={copied === message.id}
                  oncopy={copy}
                  onlink={openLink}
                />{/each}{:else if !loading}<div class="empty-room">
                <Icon name="message" />
                <h3>
                  {connected.length
                    ? 'Make the first move.'
                    : 'Ready when you are.'}
                </h3>
                <p>
                  {connected.length
                    ? 'Start the conversation with a question, a thought, or a useful link.'
                    : 'Write something now. It will be shared when another computer joins this network.'}
                </p>
              </div>{/if}
          </div>
        </section>
        {#if newMessages || hasNewer}<button
            class="new-messages"
            onclick={bottom}
            >{newMessages ? 'New messages' : 'Jump to latest'}
            <span aria-hidden="true">↓</span></button
          >{/if}
        {#if current && !room?.archived}<div class="composer-area">
            <form
              class="composer"
              onsubmit={(e) => {
                e.preventDefault();
                void send();
              }}
            >
              <textarea
                bind:this={input}
                bind:value={content}
                class:code-input={code}
                rows="2"
                placeholder={'Message ' + (room?.name ?? 'General')}
                aria-label={'Message ' + (room?.name ?? 'General')}
                oninput={persistDraft}
                onkeydown={composerKey}
                disabled={sending || loading}></textarea>
              <div class="composer-toolbar">
                <div class="format-tools">
                  <button
                    class="icon-button"
                    type="button"
                    aria-label={code
                      ? 'Switch to plain text'
                      : 'Switch to code snippet'}
                    aria-pressed={code}
                    title="Write a code snippet"
                    onclick={() => {
                      code = !code;
                      persistDraft();
                      input.focus();
                    }}><Icon name="code" /></button
                  ><button
                    class="icon-button"
                    type="button"
                    aria-label="Insert a link"
                    title="Insert a link"
                    onclick={insertLink}><Icon name="link" /></button
                  >
                </div>
                <div class="composer-meta">
                  <span class="mode-label"
                    >{code ? 'Code snippet' : 'Everyone can see this'}</span
                  ><button
                    class="send-button"
                    type="submit"
                    disabled={!content.trim() || sending || !room}
                    >{sending ? 'Saving…' : 'Send'}<Icon name="arrow" /></button
                  >
                </div>
              </div>
            </form>
            <div class="composer-footer">
              <span
                ><Icon name="globe" />Shared with everyone on this network</span
              ><span class="keyboard-hint"
                >{code
                  ? '⌘/Ctrl + Enter to send'
                  : 'Enter to send · Shift + Enter for a new line'}</span
              >
            </div>
          </div>{:else}<div class="read-only">
            {current
              ? 'Archived rooms keep their history.'
              : 'Previous network history · kept on this computer · read-only'}
          </div>{/if}
      </main>
      {#if people}<aside class="details-panel" aria-label="Nearby people">
          <div class="details-heading">
            <h2>People nearby</h2>
            <button
              class="icon-button"
              aria-label="Close people panel"
              onclick={() => (people = false)}><Icon name="x" /></button
            >
          </div>
          <p class="detail-note">
            {appState.network_detail}<br />Every room is open to everyone on
            this network.
          </p>
          <div class="detail-person">
            <span class="avatar you">{initials(appState.profile)}</span><span
              class="person-info"
              ><strong>{appState.profile} · you</strong><small
                >This computer</small
              ></span
            >
          </div>
          {#each appState.peers as peer}<div class="detail-person">
              <span class="avatar">{initials(peer.name)}</span><span
                class="person-info"
                ><strong>{peer.name}</strong><small
                  >{peer.platform} · {peer.status}</small
                ></span
              >{#if peer.status === 'connected'}<span
                  class="status-dot"
                  aria-label="Connected"
                ></span>{/if}
            </div>{/each}
          <p class="detail-caption">
            Local Chat discovers computers automatically. If none appear, allow
            Local Network access in your operating system settings and check
            that both computers use the same Wi-Fi or Ethernet network.
          </p>
          <p class="detail-caption">
            Nearby identities belong to installations. A display name does not
            verify a person's identity.
          </p>
        </aside>{/if}
    </div>
  {:else}<div class="loading">
      {desktop ? 'Opening Local Chat…' : 'Local Chat · Desktop application'}
    </div>{/if}
</div>
<dialog
  bind:this={searchDialog}
  aria-label="Search rooms and messages"
  onclick={backdrop}
  onkeydown={searchKeys}
>
  <div class="search-input-row">
    <Icon name="search" /><input
      aria-label="Search"
      bind:value={query}
      oninput={runSearch}
      placeholder={scope ? 'Search this room…' : 'Search all saved messages…'}
      autocomplete="off"
    /><button
      class="esc-key"
      aria-label="Close search"
      onclick={() => searchDialog.close()}>esc</button
    >
  </div>
  <div class="search-results">
    {#if !query && !scope}{#each roomList as r}<button
          class="search-result"
          onclick={() => {
            searchDialog.close();
            void selectRoom(r.id);
          }}
          ><Icon name="hash" /><span
            ><strong>{r.name}</strong><small
              >{r.description || 'Shared room'}</small
            ></span
          ></button
        >{/each}{/if}{#each results as result}<button
        class="search-result"
        onclick={() => choose(result)}
        ><span class="avatar">{initials(result.message.profile)}</span><span
          ><strong>{result.message.profile} in {result.room_name}</strong><small
            >{result.excerpt}</small
          ><small
            >{new Date(result.message.timestamp).toLocaleString()} · {result.current
              ? 'Current network'
              : 'Previous network · local history'}</small
          ></span
        ></button
      >{/each}{#if query && !results.length}<div class="search-empty">
        {searching ? 'Searching…' : 'No matches. Try another word.'}
      </div>{/if}
  </div>
  <div class="search-hint">
    Search includes history saved on this computer. Previous networks remain
    local.
  </div>
</dialog>
<dialog
  bind:this={roomDialog}
  aria-labelledby="room-dialog-title"
  onclick={backdrop}
>
  <div class="dialog-header">
    <h2 id="room-dialog-title">{edit ? 'Rename room' : 'Create a room'}</h2>
    <button
      class="icon-button"
      aria-label="Close"
      onclick={() => roomDialog.close()}><Icon name="x" /></button
    >
  </div>
  <form class="dialog-body" onsubmit={saveRoom}>
    {#if error}<p class="error-text" role="alert">{error}</p>{/if}
    <p>
      A shared place for a topic. Everyone on your network can see it and send
      messages.
    </p>
    <label class="field-label" for="room-name">Room name</label><input
      id="room-name"
      class="text-field"
      bind:value={roomName}
      maxlength="80"
      required
      autocomplete="off"
      placeholder="e.g. Weekend plans"
    /><label class="field-label" for="room-description"
      >Description · optional</label
    ><input
      id="room-description"
      class="text-field"
      bind:value={description}
      maxlength="240"
      placeholder="What is this room for?"
    /><span class="field-hint"
      >You can rename, archive, and restore rooms you create.</span
    >
    <div class="dialog-actions">
      <button class="button" type="button" onclick={() => roomDialog.close()}
        >Cancel</button
      ><button class="button primary" type="submit" disabled={savingRoom}
        >{edit ? 'Save changes' : 'Create room'}</button
      >
    </div>
  </form>
</dialog>
<dialog
  bind:this={settingsDialog}
  aria-labelledby="settings-title"
  onclick={backdrop}
>
  <div class="dialog-header">
    <h2 id="settings-title">Make yourself at home</h2>
    <button
      class="icon-button"
      aria-label="Close settings"
      onclick={() => settingsDialog.close()}><Icon name="x" /></button
    >
  </div>
  <form class="dialog-body" onsubmit={saveSettings}>
    {#if error}<p class="error-text" role="alert">{error}</p>{/if}
    <label class="field-label" for="display-name">Display name</label><input
      id="display-name"
      class="text-field"
      bind:value={profile}
      maxlength="80"
      required
    />
    <div class="field-label">Appearance</div>
    <div class="theme-options">
      {#each ['system', 'light', 'dark'] as choice}<button
          class="theme-option"
          class:selected={theme === choice}
          type="button"
          aria-pressed={theme === choice}
          onclick={() => {
            if (choice === 'system' || choice === 'light' || choice === 'dark')
              theme = choice;
          }}
          ><Icon
            name={choice === 'system'
              ? 'monitor'
              : choice === 'light'
                ? 'sun'
                : 'moon'}
          />{choice[0]?.toUpperCase()}{choice.slice(1)}</button
        >{/each}
    </div>
    <label class="checkbox-row"
      ><input type="checkbox" bind:checked={notifications} />Notify me about new
      messages when Local Chat is in the background</label
    >
    <p class="settings-note">
      Allow Local Network access when your operating system asks. No internet or
      account is needed. Closing this window keeps Local Chat receiving. Choose
      Quit from the app menu to stop.
    </p>
    <div class="dialog-actions">
      <button
        class="button"
        type="button"
        onclick={() => settingsDialog.close()}>Cancel</button
      ><button class="button primary" type="submit">Save changes</button>
    </div>
  </form>
</dialog>
{#if toast}<div class="toast visible" role="status">
    <Icon name="check" />{toast}
  </div>{/if}
