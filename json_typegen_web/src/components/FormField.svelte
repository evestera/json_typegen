<script lang="ts">
    import IconButton from "./IconButton.svelte";
    import XCircleIcon from "../icons/XCircleIcon.svelte";
    import Column from "./Column.svelte";
    import Row from "./Row.svelte";
    import QuestionMarkCircleIcon from "../icons/QuestionMarkCircleIcon.svelte";

    export let id: string;
    export let label: string | undefined = undefined;
    export let helpText: string | undefined = undefined;
    export let error: string | null | undefined = undefined;
    let showHelpText = false;
</script>

<div class="wrapper">
    <Row>
        <Column>
            <div style="display: flex">
                {#if label}
                    <label for={id}>{label}</label>
                {/if}
                {#if helpText}
                    <IconButton
                        on:click={() => (showHelpText = !showHelpText)}
                        style="margin-left: 3px"
                    >
                        {#if showHelpText}
                            <XCircleIcon />
                        {:else}
                            <QuestionMarkCircleIcon />
                        {/if}
                    </IconButton>
                {/if}
                {#if $$slots.afterLabel}
                    <slot name="afterLabel" />
                {/if}
            </div>
        </Column>
        {#if $$slots.toolbar}
            <Column>
                <div class="toolbar">
                    <slot name="toolbar" />
                </div>
            </Column>
        {/if}
    </Row>
    {#if showHelpText}
        <p class="help-text">{helpText}</p>
    {/if}
    {#if $$slots.default}
        <div class="slot-wrapper">
            <slot />
        </div>
    {/if}
    {#if error}
        <div class="error">{error}</div>
    {/if}
</div>

<style>
    .wrapper {
        margin: 8px 0;
    }
    .help-text {
        margin: 0 0 5px;
        font-size: 14px;
        line-height: 18px;
        max-width: 450px;
    }
    .error {
        color: red;
    }
    label {
        font-weight: 700;
        font-size: 20px;
        display: block;
        margin-bottom: 9px;
        font-family: var(--heading-font);
    }
    .toolbar {
        display: flex;
        justify-content: flex-end;
        column-gap: 3px;
        text-align: right;
    }
    .slot-wrapper {
        border-radius: 8px;
        box-shadow: 5px 5px 13px 4px #015b3440;
    }
</style>
