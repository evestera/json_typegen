# Configuration

## Name and visibility

Wherever the name of the root output type can be given you can prefix it with a
visibility specifier. E.g. `pub Point` rather than `Point`. The subtypes and
fields will "inherit" this visibility specifier. The default visibility is `pub`.

## Options block

All the interfaces can take an "options block" to configure the format
of the output. E.g. if you want to restrict the visibility of the fields, you
can use the options block `{ field_visibility: "pub(crate)" }`

The options that can be set in this manner are:

- General options:
    - `property_name_format`: Use a specific format for all properties, to
      avoid having to rename each property separately.
      Using `rename_all` with Serde and `JsonNaming` with Jackson.
      See the web interface for the available variants.
    - `unwrap`: For "unwrapping" wrapped JSON nodes before generating types.
      Combined with inference hints specifying an opaque type this allows
      creating types for wrappers and actual content separately.
      Takes a [JSON Pointer], with `-` functioning as a wildcard.
      See the [separate section below](#unwrap--wrapper-types)
- Rust-specific options:
    - `derives`: Which traits the type should derive
    - `field_visiblity`: Visibility specifier for fields
    - `deny_unknown_fields`: See [serde docs](https://serde.rs/container-attrs.html#serdedenyunknownfields)
    - `use_default_for_missing_fields`: See [serde docs](https://serde.rs/container-attrs.html#serdedefault)
        for `#[serde(default)]`
    - `allow_option_vec`: Whether the inference should allow the type
        `Option<Vec<...>>` to be inferred, or if it should be collapsed to just
        `Vec<...>`

### Field options / inference hints

In addition to these options, specific fields can be configured with their own
options block using a [JSON Pointer], with `-` functioning as a wildcard.
The options that can be set in this manner are:

- `use_type`: A string which should override the inferred type. In particular,
    the string `"map"` can be used to indicate that an object should be
    inferred and deserialized as a `HashMap<String, ...>`. Other strings are
    treated as opaque types.

[JSON Pointer]: https://tools.ietf.org/html/rfc6901

### Example

```
{
  "property_name_format": "PascalCase",
  "/-/request/headers": {
    "use_type": "map"
  }
}
```

## Unwrap / wrapper types

Often an API can have a stable wrapper type with a varying inner type,
so that it makes sense to generate/write the type for the wrapper once,
and reuse that type.
In these cases you then mostly want to only generate a type for the inner type.
To do this you can use the `unwrap` option.

To generate the outer type you can use an inference hint with an opaque type
filling in for a type parameter.
The code generation will not actually create generic types at the moment,
so you may have to manually edit the generated code afterwards.
