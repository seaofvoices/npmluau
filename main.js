#!/usr/bin/env node

const process = require('process')

const { createCLI } = require('./src/cli')

const run = createCLI()

run(process.argv)
