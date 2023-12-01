const fs = require('fs/promises')
const path = require('path')

const { reexport } = require('../luau-types-re-export/pkg')
const log = require('./log')

const SELF_NAME = 'npmluau'

const SUPPORTED_EXTENSIONS = [
  '.json',
  '.json5',
  '.toml',
  '.yml',
  '.yaml',
  '.toml',
  '.txt',
]
const RE_EXPORT_EXTENSIONS = ['.lua', '.luau']

const extractEntryPoint = async (contentPath, parentPath, level = 1) => {
  if (contentPath.startsWith('.')) {
    log.trace(`skipping '${contentPath}'`)
    return null
  }

  const fullContentPath = path.join(parentPath, contentPath)

  if (contentPath.startsWith('@')) {
    log.trace(`found scope entry '${contentPath}'`)
    return await fs
      .readdir(fullContentPath)
      .catch((_) => {
        return []
      })
      .then((content) =>
        Promise.all(
          content.flatMap((scopeContentPath) =>
            extractEntryPoint(scopeContentPath, fullContentPath, level + 1)
          )
        )
      )
      .then((entries) => entries.flat().filter((package) => !!package))
  }

  const packagePath = path.join(fullContentPath, 'package.json')
  const packageContent = await fs.readFile(packagePath).catch((_err) => {
    return null
  })

  if (packageContent === null) {
    log.trace(`skipping '${contentPath}' (no package.json)`)
    return null
  }

  let packageName
  let packageMain
  try {
    const { name = null, main = null } = JSON.parse(packageContent)
    packageName = name
    packageMain = main
  } catch (err) {
    log.warn(`unable to parse package definition at \`${packagePath}\`: ${err}`)
    return null
  }

  if (packageMain === null) {
    log.trace(`skipping '${contentPath}' (no package name and main defined)`)
    return null
  } else if (packageName === SELF_NAME) {
    log.trace(`skipping '${contentPath}'`)
    return null
  }
  log.trace(
    `found package '${packageName}' with entry point at '${packageMain}' (${fullContentPath})`
  )

  const entryPoint = path.join(packageName, packageMain)

  const entryPointFullPath = path.join(fullContentPath, packageMain)

  const entryPointContent = await fs
    .readFile(entryPointFullPath)
    .then((content) => content.toString())
    .catch((_err) => {
      return null
    })

  if (entryPointContent === null) {
    log.trace(`unable to read entry point at '${entryPointFullPath}`)
    return null
  }

  const relativeRequirePath = path
    .join(...Array(level).fill('..'), entryPoint)
    .replace(/\\/g, '/')

  let reExportedEntryPoint = null
  try {
    if (
      RE_EXPORT_EXTENSIONS.some((extension) => entryPoint.endsWith(extension))
    ) {
      reExportedEntryPoint = reexport(relativeRequirePath, entryPointContent)
    } else if (
      SUPPORTED_EXTENSIONS.some((extension) => entryPoint.endsWith(extension))
    ) {
      reExportedEntryPoint = `return require("${relativeRequirePath}")`
    } else {
      return null
    }
  } catch (err) {
    log.warn(
      `  unable to re-export types for \`${packageName}\` from \`${entryPointFullPath}\`: ${err}`
    )
    return null
  }
  return {
    name: packageName,
    entryPoint: reExportedEntryPoint,
  }
}

const extractAllEntryPoints = async (nodeModulesPath) => {
  const content = await fs.readdir(nodeModulesPath).catch((_) => {
    return []
  })

  const entryPoints = await Promise.all(
    content.map((contentPath) =>
      extractEntryPoint(contentPath, nodeModulesPath)
    )
  ).then((packages) => packages.flat().filter((package) => !!package))

  return entryPoints
}

const extractEntryPointsToFile = async (
  nodeModulesPath,
  { output, extension }
) => {
  const aliases = await extractAllEntryPoints(nodeModulesPath)
  log.info(
    `extracted ${aliases.length} entry point${aliases.length > 1 ? 's' : ''}`
  )

  try {
    await fs.rm(output, { recursive: true })
  } catch (_) {
    // catch error
  }

  try {
    await fs.mkdir(output)
  } catch (_) {
    // catch error
  }

  await Promise.all(
    aliases.map(async ({ name, entryPoint }) => {
      const fileEntryPoint = path.join(output, name + '.' + extension)

      log.debug(`about to write entry point '${fileEntryPoint}'`)

      await fs
        .mkdir(path.dirname(fileEntryPoint), { recursive: true })
        .catch((_) => {})

      return await fs.writeFile(fileEntryPoint, entryPoint).catch((err) => {
        log.warn(
          `unable to write require redirection file at \`${fileEntryPoint}\` for \`${name}\`: ${err}`
        )
      })
    })
  )
}

module.exports = {
  extractAllEntryPoints,
  extractEntryPointsToFile,
}
