const LogLevel = {
  Error: 'Error',
  Warn: 'Warn',
  Info: 'Info',
  Debug: 'Debug',
  Trace: 'Trace',
}

const LogLevelValue = {
  Error: 4,
  Warn: 3,
  Info: 2,
  Debug: 1,
  Trace: 0,
}
const LevelValueToName = Object.fromEntries(
  Object.entries(LogLevelValue).map(([key, value]) => [value, key])
)

const DEFAULT_LEVEL_FILTER = LogLevel.Warn
const DEFAULT_LEVEL_FILTER_VALUE = LogLevelValue[DEFAULT_LEVEL_FILTER]

const staticLogger = {
  levelFilter: DEFAULT_LEVEL_FILTER_VALUE,
  log: (level, ...args) => {
    if (level >= staticLogger.levelFilter) {
      staticLogger.innerLog(LevelValueToName[level], ...args)
    }
  },
  innerLog: (level, ...args) => {
    console.log(`[${level.toUpperCase()}] >`, ...args)
  },
}

const setLevelFilter = (levelFilter) => {
  if (LogLevelValue[levelFilter] === undefined) {
    console.error(`cannot assign log level filter to '${levelFilter}'`)
  }
  staticLogger.levelFilter = LogLevelValue[levelFilter]
}

const setLogImpl = (logFn) => {
  staticLogger.innerLog = logFn
}

const log = (level, ...args) => {
  staticLogger.log(LogLevelValue[level] ?? DEFAULT_LEVEL_FILTER_VALUE, ...args)
}

const logError = (...args) => {
  staticLogger.log(LogLevelValue.Error, ...args)
}

const logWarn = (...args) => {
  staticLogger.log(LogLevelValue.Warn, ...args)
}

const logInfo = (...args) => {
  staticLogger.log(LogLevelValue.Info, ...args)
}

const logDebug = (...args) => {
  staticLogger.log(LogLevelValue.Debug, ...args)
}

const logTrace = (...args) => {
  staticLogger.log(LogLevelValue.Trace, ...args)
}

module.exports = {
  LogLevel,
  setLevelFilter,
  setLogImpl,
  log,
  error: logError,
  warn: logWarn,
  info: logInfo,
  debug: logDebug,
  trace: logTrace,
}
