import {polyfill} from './polyfill';
import {$} from "./util";
import {updateDownloadLink} from "./download";
import {restoreParams, storeParams} from "./localstorage";

restoreParams();

polyfill().then(() => import("../../json_typegen_wasm/pkg")).then(typegen_wasm => {
  const render = () => {
    const typename = $('typename').value;
    let input = $('input').value;
    if (input === '') {
      input = '{}';
    }
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
      extraoptions: extraoptions_json
    })

    const combinedOptions = Object.assign({}, options, extraoptions || {});

    const result = typegen_wasm.run(typename, input, JSON.stringify(combinedOptions));

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

  render();
});
