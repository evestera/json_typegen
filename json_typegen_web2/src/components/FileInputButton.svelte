<script>
    import Button from "./Button.svelte";
</script>

<label>
    <input type="file" accept="application/json" on:change />
    <Button purelyVisual={true}><slot name="icon" slot="icon" /><slot /></Button
    >
</label>

<style>
    label {
        position: relative;
        cursor: pointer;
        overflow: hidden;
        user-select: none;
    }
    input {
        position: absolute;
        top: 0;
        left: 0;
        opacity: 0;
        width: 100%;
        height: 100%;
        cursor: pointer;
        z-index: -1;
    }
</style>
