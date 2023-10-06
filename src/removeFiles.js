const fs = require('fs/promises')
const walkdir = require('walkdir')

const removeFiles = async (rootPath, filterFn) => {
  const content = await walkdir.async(rootPath)

  await Promise.all(
    content
      .filter((filePath) => !filterFn(filePath))
      .map((filePath) => {
        return fs.rm(filePath)
      })
  )
}

module.exports = {
  removeFiles,
}
