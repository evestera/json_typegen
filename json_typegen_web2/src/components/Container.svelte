<div>
    <slot />
</div>

<style>
    div {
        max-width: 1600px;
        margin: auto;
        padding: 20px;
    }
    @media (max-width: 600px) {
        div {
            padding: 5px;
        }
    }
</style>
