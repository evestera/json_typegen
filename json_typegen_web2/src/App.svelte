<script lang="ts">
    import type { WorkerMessageType } from "./lib/WorkerMessage";
    import { WorkerMessage } from "./lib/WorkerMessage";
    import { storeParams, restoreParams } from "./lib/localstorage";
    import type { DownloadLinkProps } from "./lib/download";
    import { getDownloadLinkProps } from "./lib/download";
    import Column from "./components/Column.svelte";
    import Container from "./components/Container.svelte";
    import Row from "./components/Row.svelte";
    import Input from "./components/Input.svelte";
    import Textarea from "./components/Textarea.svelte";
    import Select from "./components/Select.svelte";
    import Checkbox from "./components/Checkbox.svelte";
    import type { Example } from "./examples/examples";
    import { examples } from "./examples/examples";
    import { readFileAsString } from "./lib/file";
    import Spinner from "./components/Spinner.svelte";
    import FormField from "./components/FormField.svelte";
    import Button from "./components/Button.svelte";
    import CodeBracketIcon from "./icons/CodeBracketIcon.svelte";
    import ArrowDownTrayIcon from "./icons/ArrowDownTrayIcon.svelte";
    import ArrowUpTrayIcon from "./icons/ArrowUpTrayIcon.svelte";
    import FileInputButton from "./components/FileInputButton.svelte";
    import ClipboardDocumentIcon from "./icons/ClipboardDocumentIcon.svelte";
    import HighlightedCode from "./components/HighlightedCode.svelte";

    const worker = new Worker(new URL("./lib/worker.ts", import.meta.url), {
        type: "module",
    });
    let workerReady = false;
    let queued: WorkerMessageType | undefined = undefined;
    let isLoading = false;

    function requestCodegen(message: WorkerMessageType) {
        if (workerReady) {
            worker.postMessage(message);
            workerReady = false;
            isLoading = true;
        } else {
            queued = message;
        }
    }

    const storedParams = restoreParams();
    let input = storedParams.input || "";
    let output = "";
    let inputMode = storedParams.options?.input_mode || "json";
    let outputMode = storedParams.options?.output_mode || "kotlin/jackson";
    let typeName = storedParams.typename || "MyRoot";
    let propertyNameFormat = storedParams.options?.property_name_format || "";
    let importStyle = storedParams.options?.import_style || "assume_existing";
    let unwrapPath = storedParams.options?.unwrap || "";
    let inferMapThreshold = storedParams.options?.infer_map_threshold || "";
    let collectAdditional = storedParams.options?.collect_additional || false;
    let extraOptions = storedParams.extraoptions || "";
    let extraOptionsError: string | null = null;
    let isLoadingLargeFile = false;
    let largeFileMessage: string | null = null;

    let downloadLink: DownloadLinkProps = getDownloadLinkProps(
        output,
        typeName,
        outputMode,
    );

    const conditionalOptions = {
        propertynameformat: ["rust", "kotlin/jackson", "python"],
        importstyle: ["rust", "kotlin/jackson", "kotlin/kotlinx", "python"],
        collectadditional: ["rust", "kotlin/jackson"],
    };

    worker.onmessage = (messageEvent) => {
        const message: WorkerMessageType = messageEvent.data;
        if (message.type === WorkerMessage.CODEGEN_COMPLETE) {
            output = message.result.trim();

            downloadLink = getDownloadLinkProps(
                output,
                message.typename,
                message.options["output_mode"],
            );
            isLoading = false;
        } else if (message.type === WorkerMessage.LOAD_FILE_COMPLETE) {
            isLoadingLargeFile = false;
            render();
        } else if (message.type === WorkerMessage.WASM_READY) {
            // no action needed
            console.log("Worker ready");
        } else {
            console.warn("Unknown worker message ", messageEvent);
        }

        // If we got a message (any message) from the worker, it is ready
        // (we don't have any kind of queue at the worker side)
        workerReady = true;
        if (queued) {
            requestCodegen(queued);
            queued = undefined;
        }
    };

    const render = () => {
        const options = {
            output_mode: outputMode,
            input_mode: inputMode,
            property_name_format: propertyNameFormat,
            import_style: importStyle,
            unwrap: unwrapPath,
            collect_additional: !!collectAdditional,
            infer_map_threshold: inferMapThreshold,
        };
        storeParams({
            typename: typeName,
            input: input.length < 1000000 ? input : "",
            options,
            extraoptions: extraOptions,
        });

        let parsedExtraOptions = {};
        extraOptionsError = null;
        try {
            parsedExtraOptions = JSON.parse(extraOptions.trim() || "{}");
        } catch (e) {
            extraOptionsError = "Invalid JSON";
        }

        const combinedOptions = Object.assign({}, options, parsedExtraOptions);

        const message: WorkerMessageType = {
            type: WorkerMessage.CODEGEN,
            typename: typeName,
            input: input || "{}",
            options: combinedOptions,
        };
        requestCodegen(message);
    };
    render();
    $: {
        typeName;
        input;
        outputMode;
        inputMode;
        propertyNameFormat;
        importStyle;
        collectAdditional;
        unwrapPath;
        inferMapThreshold;
        extraOptions;
        render();
    }

    const loadExample = (example: Example) => {
        input = example.json;
        typeName = example.typeName;
        propertyNameFormat = example.propertyNameFormat || "";
    };

    let filename = "";
    const loadFile = (event: Event) => {
        const file = (event.target as HTMLInputElement).files?.[0];
        if (!file) {
            return;
        }
        filename = file.name;
        if (file.size > 500_000) {
            input = "";
            isLoadingLargeFile = true;
            const fileSizeMb = (file.size / 1_000_000).toFixed(2);
            largeFileMessage = `"${file.name}" (${fileSizeMb} MB)`;
            worker.postMessage({
                type: WorkerMessage.LOAD_FILE,
                file,
            });
            workerReady = false;
        } else {
            readFileAsString(file).then((contents: string) => {
                input = contents;
            });
        }
    };

    const clearFile = () => {
        input = "";
        largeFileMessage = null;
        isLoadingLargeFile = false;
        worker.postMessage({
            type: WorkerMessage.CLEAR_FILE,
        });
    };

    const formatInput = () => {
        try {
            input = JSON.stringify(JSON.parse(input), null, 2);
        } catch (e) {
            // ignore
        }
    };

    const copyOutput = () => {
        navigator.clipboard.writeText(output);
    };

    const outputModeToLanguage = (mode: string) => {
        switch (mode) {
            case "rust":
                return "rust";
            case "typescript":
            case "typescript/typealias":
            case "zod":
                return "typescript";
            case "kotlin/jackson":
            case "kotlin/kotlinx":
                return "kotlin";
            case "python":
                return "python";
            case "json_schema":
            case "shape":
                return "json";
            default:
                return "plaintext";
        }
    };
</script>

<Container>
    <h1>typegen</h1>
    <p>Generate types (Rust, TS, Kotlin, Python, ...) from JSON samples or SQL</p>
    <p>
        Examples:
        {#each examples as example}
            <a
                href="#?example={example.id}"
                on:click={() => loadExample(example)}>{example.name}</a
            >{" "}
        {/each}
    </p>

    <Row>
        <Column>
            <FormField id="typename" label="Type name">
                <Input id="typename" bind:value={typeName} />
            </FormField>

            <FormField id="input" label="Input code">
                <svelte:fragment slot="toolbar">
                    <Button on:click={formatInput}
                        ><CodeBracketIcon slot="icon" /> Format JSON</Button
                    >
                    <FileInputButton on:change={loadFile}
                        ><ArrowUpTrayIcon slot="icon" />
                        {filename || "Load from file..."}</FileInputButton
                    >
                </svelte:fragment>

                <div class="overlay-container">
                    {#if largeFileMessage}
                        <div id="large-file-overlay">
                            <p>Large input file not shown</p>
                            <p id="large-file-message">{largeFileMessage}</p>
                            <button on:click={clearFile}
                                >Clear input file</button
                            >
                            <br />
                            {#if isLoadingLargeFile}
                                <Spinner />
                            {/if}
                        </div>
                    {/if}

                    <Textarea id="input" bind:value={input} rows={15} />
                </div>
            </FormField>

            <FormField id="inputmode" label="Input language">
                <Select
                    id="inputmode"
                    bind:value={inputMode}
                    options={[
                        ["json", "JSON"],
                        ["sql", "SQL (create table statement)"],
                    ]}
                />
            </FormField>

            <FormField id="outputmode" label="Output language">
                <Select
                    id="outputmode"
                    bind:value={outputMode}
                    options={[
                        ["rust", "Rust"],
                        ["typescript", "Typescript"],
                        [
                            "typescript/typealias",
                            "Typescript (single typealias)",
                        ],
                        ["kotlin/jackson", "Kotlin (Jackson)"],
                        ["kotlin/kotlinx", "Kotlin (kotlinx.serialization)"],
                        ["python", "Python (pydantic)"],
                        ["json_schema", "JSON Schema"],
                        ["zod", "Zod Schema"],
                        ["shape", "Shape (internal representation)"],
                    ]}
                />
            </FormField>

            {#if conditionalOptions.propertynameformat.includes(outputMode)}
                <FormField id="propertynameformat" label="Property name format">
                    <Select
                        id="propertynameformat"
                        bind:value={propertyNameFormat}
                        options={[
                            ["", "-"],
                            ["PascalCase", "PascalCase"],
                            ["camelCase", "camelCase"],
                            ["snake_case", "snake_case"],
                            ["SCREAMING_SNAKE_CASE", "SCREAMING_SNAKE_CASE"],
                            ["kebab-case", "kebab-case"],
                            ["SCREAMING-KEBAB-CASE", "SCREAMING-KEBAB-CASE"],
                            ["UPPERCASE", "UPPERCASE"],
                        ]}
                    />
                </FormField>
            {/if}

            {#if conditionalOptions.importstyle.includes(outputMode)}
                <FormField id="importstyle" label="Import style">
                    <Select
                        id="importstyle"
                        bind:value={importStyle}
                        options={[
                            ["assume_existing", "Assume existing imports"],
                            ["add_imports", "Add needed imports"],
                            ["qualified_paths", "Use fully qualified paths"],
                        ]}
                    />
                </FormField>
            {/if}

            {#if conditionalOptions.collectadditional.includes(outputMode)}
                <FormField
                    id="collectadditional"
                    label="Collect additional properties"
                >
                    <svelte:fragment slot="afterLabel">
                        <Checkbox
                            id="collectadditional"
                            bind:value={collectAdditional}
                        />
                    </svelte:fragment>
                </FormField>
            {/if}

            <FormField
                id="unwrappath"
                label="Unwrap path"
                helpText={"JSON Pointer to the element you want to generate a type for, skipping wrapping elements. " +
                    "- is used as a wildcard. " +
                    'E.g. in the example "List of Magic cards", /cards/- would ignore the wrapper object and array'}
            >
                <Input id="unwrappath" bind:value={unwrapPath} />
            </FormField>

            <FormField
                id="extraoptions"
                label="Extra options"
                error={extraOptionsError}
                helpText="Add extra configuration / hints for the inference. E.g. what type to use for a specific field."
            >
                <Textarea
                    id="extraoptions"
                    bind:value={extraOptions}
                    rows={10}
                />
            </FormField>

            <FormField
                id="infermapthreshold"
                label="Infer map threshold"
                helpText={"If the number of keys in a map is less than this threshold, the map will be inferred as a struct. " +
                    "Otherwise, it will be inferred as a map. " +
                    "If blank, structs will always be used."}
            >
                <Input id="infermapthreshold" bind:value={inferMapThreshold} />
            </FormField>
        </Column>

        <Column>
            <FormField id="output" label="Output">
                <svelte:fragment slot="toolbar">
                    <Button on:click={copyOutput}
                        ><ClipboardDocumentIcon slot="icon" /> Copy to clipboard</Button
                    >
                    <Button
                        href={downloadLink.href}
                        download={downloadLink.download}
                    >
                        <ArrowDownTrayIcon slot="icon" /> Download as file
                    </Button>
                </svelte:fragment>
                <HighlightedCode
                    language={outputModeToLanguage(outputMode)}
                    code={output}
                />
            </FormField>
        </Column>
    </Row>
</Container>
