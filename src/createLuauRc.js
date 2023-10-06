const fs = require('fs/promises')
const path = require('path')

const NOCHECK_CONFIG = JSON.stringify(
  {
    languageMode: 'nocheck',
  },
  null,
  4
)

const createNoCheckLuauRc = async (parentPath) => {
  const filePath = path.join(parentPath, '.luaurc')

  await fs.writeFile(filePath, NOCHECK_CONFIG)
}

module.exports = {
  createNoCheckLuauRc,
}
