const { Command } = require('commander')
const path = require('path')

const { extractEntryPointsToFile } = require('./extractEntryPoints')
const { removeFiles } = require('./removeFiles')
const { createNoCheckLuauRc } = require('./createLuauRc')

const CLI_VERSION = '0.1.0'

const createCLI = () => {
  const program = new Command()

  program
    .name('npmluau')
    .description('a utility to manage Luau npm dependencies')
    .version(CLI_VERSION)
    .option('--target <target>', 'the path where the Luau package is defined')
    .option(
      '--extension <extension>',
      'the file extension to use when generating module links (default is `luau`)'
    )
    .option(
      '--keep-rojo-configs',
      "when specified, the tool will not delete Rojo files (matching '*.project.json')"
    )
    .option(
      '--keep-luaurc',
      "when specified, the tool will not delete Luau config files (named '.luaurc')"
    )
    .action(
      async ({
        target = '.',
        extension = 'luau',
        keepLuaurc = false,
        keepRojoConfigs = false,
      }) => {
        const projectNodeModules = path.join(target, 'node_modules')
        const output = path.join(projectNodeModules, '.luau-aliases')

        await extractEntryPointsToFile(projectNodeModules, {
          output,
          extension,
        })

        if (!keepLuaurc || !keepRojoConfigs) {
          await removeFiles(projectNodeModules, (filePath) => {
            const baseName = path.basename(filePath)

            return (
              (keepRojoConfigs || !baseName.endsWith('.project.json')) &&
              (keepLuaurc || baseName !== '.luaurc')
            )
          })
        }

        await createNoCheckLuauRc(projectNodeModules)
      }
    )

  return (args) => {
    program.parse(args)
  }
}

module.exports = {
  createCLI,
}
