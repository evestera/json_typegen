<script lang="ts">
    export let id: string;
    export let value: string = "";
</script>

<input {id} type="text" bind:value on:change on:keyup />

<style>
    input {
        font-family: var(--body-font);
        color: var(--text);
        font-size: 14px;
        padding: var(--px1);
        border: 1px solid #ccc;
        border-radius: 8px;
        background-color: rgba(255, 255, 255, 0.8);
        width: calc(100% - var(--px2));
    }
</style>
