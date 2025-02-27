<div>
    <slot />
</div>

<style>
    div {
        display: flex;
        flex-direction: row;
        column-gap: var(--px2);
    }
    @media (max-width: 720px) {
        div {
            flex-direction: column;
        }
    }
</style>
