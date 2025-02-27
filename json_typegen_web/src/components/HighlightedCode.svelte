<script lang="ts">
    import { getHighlighter, setCDN, renderToHtml } from "shiki";

    export let code: string;
    export let language: string;

    setCDN("/shiki/");
    const highlighter = getHighlighter({
        theme: "github-light",
        langs: ["rust", "typescript", "kotlin", "python", "json"],
    });

    let highlighted = "";

    $: {
        highlighter.then((highlighter) => {
            const tokens = highlighter.codeToThemedTokens(code, language);
            highlighted = renderToHtml(tokens, {
                bg: "rgba(255, 255, 255, 0.8)",
            });
        });
    }
</script>

{@html highlighted}

<style>
    :global(pre) {
        margin: 0;
        color: var(--text);
        font-size: 14px;
        padding: var(--px1);
        border: 1px solid #ccc;
        border-radius: 8px;
        background-color: rgba(255, 255, 255, 0.8);
        width: calc(100% - var(--px2));
        white-space: pre-wrap;
    }
    :global(pre code) {
        font-family: var(--mono-font);
    }
</style>
