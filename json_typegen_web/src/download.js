import {$} from "./util";

function createFilename(typename, output_mode) {
    const extensions = {
        "rust": "rs",
        "typescript": "ts",
        "typescript/typealias": "ts",
        "kotlin/jackson": "kt",
        "kotlin/kotlinx": "kt",
        "python": "py",
        "json_schema": "json",
        "shape": "json",
    }
    return typename + "." + extensions[output_mode];
}

let objectUrl;

export function updateDownloadLink(result, typename, options) {
    if (objectUrl) {
        URL.revokeObjectURL(objectUrl);
    }
    const blob = new Blob([result], {type: "text/plain"});
    objectUrl = URL.createObjectURL(blob);
    const a = $('filedownload');
    a.href = objectUrl;
    a.download = createFilename(typename, options.output_mode);
}
