<script lang="ts">
    export let id: string;
    export let value: boolean = false;
</script>

<input {id} type="checkbox" bind:checked={value} on:change />

<style>
    input {
        width: 18px;
        height: 18px;
        margin-left: 10px;
    }
</style>
