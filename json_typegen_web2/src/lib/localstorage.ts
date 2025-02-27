type StoredParams = {
  typename?: string;
  input?: string;
  options?: {
    input_mode?: string;
    output_mode?: string;
    property_name_format?: string;
    unwrap?: string;
    import_style?: string;
    collect_additional?: boolean;
    infer_map_threshold?: string;
  };
  extraoptions?: string;
};

export function storeParams(params: StoredParams) {
  localStorage.setItem("json_typegen_params", JSON.stringify(params));
}

export function restoreParams(): StoredParams {
  try {
    let params_json = localStorage.getItem("json_typegen_params");
    return (params_json && JSON.parse(params_json)) || {};
  } catch (e) {
    console.error(e);
  }
  return {};
}
