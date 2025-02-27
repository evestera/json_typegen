<script lang="ts">
    export let id: string;
    export let value: string = "";
    export let options: [string, string][] = [];
</script>

<select {id} bind:value on:change>
    {#each options as option}
        <option value={option[0]}>{option[1]}</option>
    {/each}
</select>

<style>
    select {
        font-family: var(--body-font);
        color: var(--text);
        font-size: 14px;
        padding: var(--px1);
        border: 1px solid #ccc;
        border-radius: 8px;
        background-color: rgba(255, 255, 255, 0.8);
        width: 100%;
    }
</style>
