[![checks](https://github.com/seaofvoices/npmluau/actions/workflows/test.yml/badge.svg)](https://github.com/seaofvoices/npmluau/actions/workflows/test.yml)
![version](https://img.shields.io/github/package-json/v/seaofvoices/npmluau)
[![GitHub top language](https://img.shields.io/github/languages/top/seaofvoices/npmluau)](https://github.com/luau-lang/luau)
![license](https://img.shields.io/npm/l/npmluau)
![npm](https://img.shields.io/npm/dt/npmluau)

# npmluau

This utility can be used to allow Luau projects to use npm as a package manager. It lets your projects use string requires: use relative file paths for the project's files or prefix your dependency names with `@pkg` to use them.

If you are interested to know why should use npm for Luau development, read [Why You Should Use npm for Luau](https://medium.com/@jeparlefrancais/why-you-should-use-npm-for-luau-22113f54f1fa).

If you are developing on a platform that does not support requires with, you can use a tool like [darklua](https://github.com/seaofvoices/darklua) to automatically convert requires into your platform specific implementation.

## How to use

Add `npmluau` in your dev-dependencies:

```bash
yarn add --dev npmluau
```

Or if you are using `npm`:

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

**[Luau-lsp](https://github.com/JohnnyMorganz/luau-lsp)**:

If you using the VS code extension, you can define a directory alias in your `.luaurc`:

```json
{
  "aliases": {
    "pkg": "node_modules/.luau-aliases"
  }
}
```

## License

This project is available under the MIT license. See [LICENSE.txt](LICENSE.txt) for details.
