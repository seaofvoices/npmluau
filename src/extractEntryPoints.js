const fs = require('fs/promises')
const path = require('path')

const { reexport } = require('../luau-types-re-export/pkg')

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

const extractEntryPoint = async (contentPath, nodeModulesPath) => {
  if (contentPath.startsWith('.')) {
    return null
  }

  const packagePath = path.join(nodeModulesPath, contentPath, 'package.json')
  const packageContent = await fs.readFile(packagePath).catch((_err) => {
    return null
  })

  if (packageContent === null) {
    return null
  }

  let packageName
  let packageMain
  try {
    const { name = null, main = null } = JSON.parse(packageContent)
    packageName = name
    packageMain = main
  } catch (err) {
    console.warn(
      `unable to parse package definition at \`${packagePath}\`: ${err}`
    )
    return null
  }

  if (packageMain === null || packageName === SELF_NAME) {
    return null
  }

  const entryPoint = path.join(contentPath, packageMain)

  const entryPointFullPath = path.join(nodeModulesPath, entryPoint)

  const entryPointContent = await fs
    .readFile(entryPointFullPath)
    .then((content) => content.toString())
    .catch((_err) => {
      return null
    })

  if (entryPointContent === null) {
    return null
  }

  const relativeRequirePath = path.join('..', entryPoint).replace(/\\/g, '/')

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
    console.warn(
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

  const result = await Promise.all(
    content.map((contentPath) =>
      extractEntryPoint(contentPath, nodeModulesPath)
    )
  )

  const entryPoints = result.filter((package) => !!package)

  return entryPoints
}

const extractEntryPointsToFile = async (
  nodeModulesPath,
  { output, extension }
) => {
  const aliases = await extractAllEntryPoints(nodeModulesPath)

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

      return await fs.writeFile(fileEntryPoint, entryPoint).catch((err) => {
        console.warn(
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
