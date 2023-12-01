const { Command } = require('commander')
const path = require('path')
const process = require('process')

const log = require('./log')
const { extractEntryPointsToFile } = require('./extractEntryPoints')
const { removeFiles } = require('./removeFiles')
const { createNoCheckLuauRc } = require('./createLuauRc')
const fs = require('fs').promises

const CLI_VERSION = '0.1.0'

const getLevelFilterFromVerbosity = (value) => {
  switch (value) {
    case 0:
      return log.LogLevel.Warn
    case 1:
      return log.LogLevel.Info
    case 2:
      return log.LogLevel.Debug
    default:
      return log.LogLevel.Trace
  }
}

const createCLI = () => {
  const program = new Command()

  program
    .name('npmluau')
    .description('a utility to manage Luau npm dependencies')
    .version(CLI_VERSION)
    .option(
      '-v, --verbose',
      'verbosity that can be increased',
      (_, prev) => prev + 1,
      0
    )
    .option('--target <target>', 'the path where the Luau package is defined')
    .option(
      '--modules-folder <module-folder>',
      'the name of the directory where packages are installed (default is `node_modules`)'
    )
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
        verbose,
        target = '.',
        extension = 'luau',
        modulesFolder = 'node_modules',
        keepLuaurc = false,
        keepRojoConfigs = false,
      }) => {
        log.setLevelFilter(getLevelFilterFromVerbosity(verbose))
        const projectNodeModules = path.join(target, modulesFolder)

        try {
          await fs.stat(projectNodeModules)
        } catch (err) {
          log.error(`unable to find modules folder at ${projectNodeModules}`)
          process.exit(1)
        }

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
