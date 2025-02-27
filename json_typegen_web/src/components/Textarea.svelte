<script lang="ts">
    export let id: string;
    export let name = id;
    export let value: string = "";
    export let readonly: boolean = false;
    export let autosize: boolean = false;
    export let rows: number | undefined = undefined;

    let ref: HTMLElement | null = null;
    const onValueChange = () => {
        if (ref && autosize) {
            ref.style.height = "10px";
            setTimeout(() => {
                if (ref) ref.style.height = ref.scrollHeight + "px";
            }, 0);
        }
    };
    $: value && onValueChange();
</script>

<textarea
    {id}
    {name}
    bind:value
    {readonly}
    on:change
    on:keyup
    bind:this={ref}
    {rows}
></textarea>

<style>
    textarea {
        font-family: var(--mono-font);
        color: var(--text);
        font-size: 14px;
        padding: var(--px1);
        border: 1px solid #ccc;
        border-radius: 8px;
        background-color: rgba(255, 255, 255, 0.8);
        width: calc(100% - var(--px2));
        display: block;
        resize: vertical;
        min-height: 100px;
    }
</style>
