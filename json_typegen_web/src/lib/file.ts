export function readFileAsString(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      if (typeof reader.result === "string") {
        resolve(reader.result);
      } else {
        reject("FileReader result is not a string");
      }
    };
    reader.onerror = reject;
    reader.readAsText(file);
  });
}
