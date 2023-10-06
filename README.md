# npmluau

This utility can be used to allow Luau projects to use npm as a package manager. It lets your projects use string requires: use relative file paths for the project's files or prefix your dependency names with `@pkg` to use them.

If you are developing on a platform that does not support requires with, you can use a tool like [darklua](https://github.com/seaofvoices/darklua) to automatically convert requires into your platform specific implementation.

## How to use

Add `npmluau` in your dev-dependencies:

```bash
npm install --save-dev npmluau
```

In your Luau project `package.json` file, add a `prepare` script to run `npmluau`:

```json
  "scripts": {
    "prepare": "npmluau",
  }
```

## How it works

This utility will generate a folder named `.luau-aliases` inside `node_modules` after installing your dependencies that contains module links to each dependency.

_[Luau-lsp](https://github.com/JohnnyMorganz/luau-lsp)_:

If you using the VS code extension, you can define a directory alias in your workspace settings:

```json
{
  "luau-lsp.require.directoryAliases": {
    "@pkg": "node_modules/.luau-aliases"
  }
}
```

If you are also running `luau-lsp` from the command line interface, you can provide the directory aliases within a configuration file and pass it to the `--settings` argument:

```json
{
  "luau-lsp.require.mode": "relativeToFile",
  "luau-lsp.require.directoryAliases": {
    "@pkg": "node_modules/.luau-aliases"
  }
}
```
