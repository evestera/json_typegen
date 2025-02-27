<script lang="ts">
    export let purelyVisual = false;
    export let href: string | undefined = undefined;
    export let download: string | undefined = undefined;
</script>

{#if purelyVisual}
    <span class="visual-button">
        {#if $$slots.icon}
            <span class="icon"><slot name="icon" /></span>
        {/if}
        <span class="text"><slot /></span>
    </span>
{:else if href}
    <a {href} {download}>
        {#if $$slots.icon}
            <span class="icon"><slot name="icon" /></span>
        {/if}
        <span class="text"><slot /></span>
    </a>
{:else}
    <button on:click>
        {#if $$slots.icon}
            <span class="icon"><slot name="icon" /></span>
        {/if}
        <span class="text"><slot /></span>
    </button>
{/if}

<style>
    a,
    button,
    .visual-button {
        border: 0;
        border-radius: 8px;
        height: 24px;
        color: #fff;
        background: var(--accent);
        background: linear-gradient(
            306deg,
            rgba(65, 52, 131, 1) 0%,
            rgba(94, 76, 184, 1) 100%
        );
        cursor: pointer;
        display: inline-block;
        font-size: 12px;
        font-weight: 400;
        margin: 0;
        padding: 2px 14px;
        text-decoration: none;
        vertical-align: middle;
        white-space: nowrap;
        box-sizing: content-box;
        font-family: var(--body-font);
    }
    a:hover,
    button:hover,
    .visual-button:hover {
        background: var(--accent-dark);
    }
    .text {
        display: inline-block;
        line-height: 23px;
        vertical-align: baseline;
    }
    .icon {
        display: inline-block;
        vertical-align: middle;
        width: 20px;
        height: 20px;
        margin-right: 3px;
    }
</style>
