import {$} from "./util";

export function storeParams(params) {
    localStorage.setItem("json_typegen_params", JSON.stringify(params));
}

export function restoreParams() {
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
}
