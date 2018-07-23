# Configuration

## Name and visibility

Wherever the name of the root output type can be given you can prefix it with a
visibility specifier. E.g. `pub Point` rather than `Point`. The subtypes and
fields will "inherit" this visiblity specifier.

## Options block

Both the macro and the CLI can take an "options block" to configure the format
of the output. E.g. if you want to restrict the visibility of the fields, you
can use the options block `{ field_visibility: "pub(crate)" }`

The options that can be set in this manner are:

- `derives`: Which traits the type should derive
- `field_visiblity`: Visibility specifier for fields
- `deny_unknown_fields`: See [serde docs](https://serde.rs/container-attrs.html#serdedenyunknownfields)
- `use_default_for_missing_fields`: See [serde docs](https://serde.rs/container-attrs.html#serdedefault)
    for `#[serde(default)]`
- `allow_option_vec`: Whether the inference should allow the type
    `Option<Vec<...>>` to be inferred, or if it should be collapsed to just
    `Vec<...>`

In addition to these options, specific fields can be configured with their own
options block using a [JSON Pointer](https://tools.ietf.org/html/rfc6901). The
options that can be set in this manner are:

- `use_type`: A string which should override the inferred type. In particular,
    the string `"map"` can be used to indicate that an object should be
    inferred and deserialized as a `HashMap<String, ...>`. Other strings are
    treated as opaque types.
