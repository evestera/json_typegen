function createFilename(typename: string, output_mode: string): string {
  const extensions: Record<string, string> = {
    rust: "rs",
    typescript: "ts",
    "typescript/typealias": "ts",
    "kotlin/jackson": "kt",
    "kotlin/kotlinx": "kt",
    python: "py",
    json_schema: "json",
    shape: "json",
  };
  return typename + "." + extensions[output_mode];
}

let objectUrl: string | undefined = undefined;
export type DownloadLinkProps = {
  href: string;
  download: string;
};

export function getDownloadLinkProps(
  result: string,
  typename: string,
  output_mode: string,
): DownloadLinkProps {
  if (objectUrl) {
    URL.revokeObjectURL(objectUrl);
  }
  const blob = new Blob([result], { type: "text/plain" });
  objectUrl = URL.createObjectURL(blob);
  return {
    href: objectUrl,
    download: createFilename(typename, output_mode),
  };
}
