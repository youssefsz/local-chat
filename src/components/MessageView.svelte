<script lang="ts">
  import Icon from './Icon.svelte';
  import type { Message } from '../lib/desktop/api';
  let {
    message,
    own,
    copied,
    highlighted,
    oncopy,
    onlink,
  }: {
    message: Message;
    own: boolean;
    copied: boolean;
    highlighted: boolean;
    oncopy: (message: Message) => void;
    onlink: (url: string) => void;
  } = $props();
  const initials = (name: string) =>
    name
      .trim()
      .split(/\s+/)
      .slice(0, 2)
      .map((x) => x[0])
      .join('')
      .toUpperCase();
  let pieces = $derived(message.content.split(/(https?:\/\/[^\s<>"']+)/g));
  const date = (time: number) => new Date(time).toLocaleString();
</script>

<article
  class:highlight={highlighted}
  class="message"
  id={'msg-' + message.id}
  data-message={message.id}
  tabindex="-1"
>
  <span class="avatar" class:you={own}
    ><span>{initials(message.profile)}</span></span
  >
  <div class="message-content">
    <div class="message-meta">
      <span class="author">{message.profile}</span>{#if own}<span
          class="you-label">you</span
        >{/if}<time
        class="message-time"
        datetime={new Date(message.timestamp).toISOString()}
        title={date(message.timestamp)}
        >{new Date(message.timestamp).toLocaleTimeString([], {
          hour: '2-digit',
          minute: '2-digit',
        })}</time
      >
    </div>
    {#if message.mode === 'code'}
      <div class="code-block">
        <div class="code-header">
          <span><Icon name="code" small />Code</span><button
            class="code-copy"
            aria-label={copied ? 'Copied' : 'Copy code snippet'}
            onclick={() => oncopy(message)}
            ><Icon name={copied ? 'check' : 'copy'} /><span
              >{copied ? 'Copied' : 'Copy code'}</span
            ></button
          >
        </div>
        <pre><code>{message.content}</code></pre>
      </div>
    {:else}
      <p class="message-text">
        {#each pieces as piece}{#if /^https?:\/\//.test(piece)}<a
              href={piece}
              onclick={(event) => {
                event.preventDefault();
                onlink(piece);
              }}>{piece}</a
            >{:else}{piece}{/if}{/each}
      </p>
    {/if}
    {#if own}<div class="local-saved">
        <Icon
          name={message.delivery === 'shared' ? 'check' : 'monitor'}
        />{message.delivery === 'shared'
          ? 'Shared with connected peers'
          : 'Saved locally · waiting for peers'}
      </div>{/if}
  </div>
  {#if message.mode !== 'code'}<div class="message-tools">
      <button
        class="icon-button"
        aria-label={copied ? 'Copied' : 'Copy message from ' + message.profile}
        title="Copy message"
        onclick={() => oncopy(message)}
        ><Icon name={copied ? 'check' : 'copy'} /></button
      >
    </div>{/if}
</article>
