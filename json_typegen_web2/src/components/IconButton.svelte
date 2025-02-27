<button on:click style={$$props.style}>
    <slot />
</button>

<style>
    button {
        background: transparent;
        border: none;
        width: 22px;
        height: 22px;
        padding: 0;
        margin: 0;
        display: inline;
        cursor: pointer;
    }
</style>
