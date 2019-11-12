# Making a release of json_typegen

```bash
VERSION=v0.3.3
git checkout master
# update Cargo.toml files with $VERSION
git commit -am "Release $VERSION"
git tag $VERSION
cd json_typegen_shared
cargo publish
cd ../json_typegen
cargo publish
cd ../json_typegen_cli
cargo publish
cd ../json_typegen_web
npm run deploy
cd ../json_typegen_wasm/pkg
npm publish

git push origin master $VERSION
# create release on github
# binaries are built on CI and attached to the release
```
