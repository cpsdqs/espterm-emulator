const EventEmitter = require('events')
const os = require('os')
const pty = require('pty.js')

const io = require('../native')

let emitter = module.exports = new EventEmitter()

let options = {
  width: 100,
  height: 36,
  cursorKeysAppMode: false,
  numpadKeysAppMode: false,
  functionKeysMode: false,
  trackMouseClicks: false,
  trackMouseMovement: false,
  enableButtons: true,
  enableMenu: true
}

if (Number.isFinite(+process.argv[4])) options.width = process.argv[4] | 0
if (Number.isFinite(+process.argv[5])) options.height = process.argv[5] | 0

console.log(`Size: ${options.width}x${options.height}`)

let userInfo = os.userInfo()

let shell = pty.spawn(userInfo.shell, ['--login'], {
  name: 'xterm-256color',
  cols: options.width,
  rows: options.height,
  cwd: userInfo.homedir,
  env: {
    LANG: 'en_US.UTF-8',
    HOME: userInfo.homedir,
    TERM_PROGRAM: 'ESPTerm',
    TMPDIR: os.tmpdir(),
    PATH: '/usr/bin:/bin:/usr/sbin:/sbin',
    USER: userInfo.username
  }
})

let terminal = new io.Terminal(options.width, options.height)
// terminal.on('bell', () => emitter.emit('bell'))
// terminal.on('window-title', title => emitter.emit('update-title', title))

let ignoreNextBell = false
let updateShell = function (data) {
  terminal.write(data.toString())
}

let dataBufferImmediate = null
let dataBuffer = ''
shell.on('data', data => {
  dataBuffer += data
  clearImmediate(dataBufferImmediate)
  dataBufferImmediate = setImmediate(() => {
    updateShell(dataBuffer)
    dataBuffer = ''
  })
})
shell.on('close', () => {
  updateShell('\x1b[0;31;1m\n' +
    '[Exited. You should probably restart the server]\x1b[?25l')
})

emitter.updateShell = updateShell
emitter.terminal = terminal
emitter.options = options
emitter.write = (...args) => shell.write(...args)
