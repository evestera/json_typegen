import {polyfill} from './polyfill';
import {$} from "./util";
import {updateDownloadLink} from "./download";
import {restoreParams, storeParams} from "./localstorage";

restoreParams();

let largeInput;

polyfill().then(() => import("../../json_typegen_wasm/pkg")).then(typegen_wasm => {
  const render = () => {
    const typename = $('typename').value;
    let input = largeInput || $('input').value;
    const options = ({
      output_mode: $('outputmode').value,
      property_name_format: $('propertynameformat').value,
      unwrap: $('unwrap').value,
    });

    const extraoptions_elem = $('extraoptions');
    const extraoptions_json = extraoptions_elem.value;
    let extraoptions;
    try {
      extraoptions = extraoptions_json && JSON.parse(extraoptions_json);
      extraoptions_elem.classList.remove("is-danger")
    } catch (e) {
      extraoptions_elem.classList.add("is-danger")
    }

    storeParams({
      typename,
      input: (input.length < 1000000) ? input : "",
      options,
      extraoptions: extraoptions ? extraoptions_json : undefined
    })

    const combinedOptions = Object.assign({}, options, extraoptions || {});

    const result = typegen_wasm.run(typename, input || "{}", JSON.stringify(combinedOptions));

    const target = $('target');
    target.value = result.trim();
    target.style.height = "10px";
    target.style.height = (target.scrollHeight + 5) + "px";

    updateDownloadLink(result, typename, options);
  };

  $('typename').onkeyup = render;
  $('input').onkeyup = render;
  $('outputmode').onchange = render;
  $('propertynameformat').onchange = render;
  $('unwrap').onkeyup = render;
  $('extraoptions').onkeyup = render;

  $('loadfile').onchange = (event) => {
    const file = event.target.files[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = (fileEvent) => {
        const contents = fileEvent.target.result;
        if (file.size > 1000000) {
          largeInput = contents;
          $('input').value = "";
          $('large-file-overlay').classList.remove("is-invisible");
          const fileSizeMb = (file.size / 1000000).toFixed(2);
          $('large-file-message').textContent = `"${file.name}" (${fileSizeMb} MB)`
        } else {
          $('input').value = contents;
        }
        render();
      }
      reader.readAsText(file);
    }
  }

  $('clear-input-button').onclick = () => {
    largeInput = undefined;
    $('large-file-overlay').classList.add("is-invisible");
    $('input').value = "";
    render();
  }

  render();
});
