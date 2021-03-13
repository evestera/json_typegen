import {polyfill} from './polyfill';

const $ = (id) => document.getElementById(id);

function storeParams(params) {
  localStorage.setItem("json_typegen_params", JSON.stringify(params));
}

polyfill().then(() => import("../../json_typegen_wasm/pkg")).then(module => {
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
    if (extraoptions) {
      Object.assign(options, extraoptions);
    }
    storeParams({
      typename,
      input: (input.length < 1000000) ? input : "",
      options,
      extraoptions: extraoptions_json
    })
    const result = module.run(typename, input, JSON.stringify(options));
    $('target').innerHTML = result
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');
  };

  $('typename').onkeyup = render;
  $('input').onkeyup = render;
  $('outputmode').onchange = render;
  $('propertynameformat').onchange = render;
  $('unwrap').onkeyup = render;
  $('extraoptions').onkeyup = render;

  render();
});

let params;
try {
  let params_json = localStorage.getItem("json_typegen_params");
  params = params_json && JSON.parse(params_json);
} catch (e) {
  console.error(e);
}
if (params) {
  if (params.typename) {
    $('typename').value = params.typename;
  }
  if (params.input) {
    $('input').value = params.input;
  }

  if (params.options) {
    $('outputmode').value = params.options.output_mode;
    $('propertynameformat').value = params.options.property_name_format;
    $('unwrap').value = params.options.unwrap;
  }

  if (params.extraoptions) {
    $('extraoptions').value = params.extraoptions;
  }
}
