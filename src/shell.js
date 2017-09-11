const EventEmitter = require('events')
const os = require('os')
const pty = require('pty.js')
const Anser = require('./anser')

const io = require('./io')

let emitter = module.exports = new EventEmitter()

let options = {
  height: 25,
  width: 80,
  cursorHanging: false,
  cursorKeysAppMode: false,
  numpadKeysAppMode: false,
  functionKeysMode: false,
  trackMouseClicks: false,
  trackMouseMovement: false,
  enableButtons: true,
  enableMenu: true
}

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

let anser = new Anser(terminal)
anser.PALETTE_COLORS = []
for (let i = 0; i < 256; i++) anser.PALETTE_COLORS.push(i)

let ignoreNextBell = false
let updateShell = function (data) {
  data = data.toString()
  // regex lookaheads:
  // 1. OSC sequence
  // 2. Designate character set
  // 3. Special characters (newline, CR, backspace, bell)
  let slices = data.split(/(?=\x1b][^\x07]+?\x07|\x1b[()*+-./][0AB]|[\n\r\b\x07])/g)
  for (let i in slices) {
    let slice = slices[i]
    // handle special stuff

    // ignore designate character set
    if (slice.match(/^\x1b[()*+-./][0AB]/)) slice = slice.substr(3)

    // $ because the slice will end before the next \x07
    let oscSequence = slice.match(/^\x1b]([^\x07]+?)$/)
    if (oscSequence) {
      ignoreNextBell = true
      slice = slice.substr(oscSequence[0].length)
      let parts = oscSequence[1].split(';')
      let type = +parts[0]
      if (type === 0) {
        // update the title
        emitter.emit('update-title', parts[1])
      } else if (type === 50 || type === 1337 &&
          parts[1].startsWith('CursorShape')) {
        let cursorShape = parts[1].match(/\d+/) | 0
        // ESPTerm doesn't support this so... ignore
      }
    } else {
      let leadingSpecial = slice[0]
      if ('\n\r\b\x07'.includes(leadingSpecial)) slice = slice.substr(1)
      if (leadingSpecial === '\n') {
        terminal.cursorPos[0] = 0
        terminal.cursorPos[1]++
        if (terminal.cursorPos[1] > terminal.height - 1) terminal.scroll()
      } else if (leadingSpecial === '\r') {
        terminal.cursorPos[0] = 0
      } else if (leadingSpecial === '\b') {
        terminal.cursorPos[0]--
        if (terminal.cursorPos[0] < 0) terminal.cursorPos[0] = 0
      } else if (leadingSpecial === '\x07') {
        if (ignoreNextBell) ignoreNextBell = false
        else emitter.emit('bell')
      }
    }

    let parts = anser.ansiToJson(slice)
    for (let part of parts) {
      if (part.fg === null) part.fg = 7
      if (part.bg === null) part.bg = 0

      let attrs = 0
      if (part.decoration === 'bold') attrs |= 1
      if (part.decoration === 'dim') attrs |= 1 << 1
      if (part.decoration === 'italic') attrs |= 1 << 2
      if (part.decoration === 'underline') attrs |= 1 << 3
      if (part.decoration === 'blink') attrs |= 1 << 4
      if (part.decoration === 'fraktur') attrs |= 1 << 5
      if (part.decoration === 'strikethrough') attrs |= 1 << 6
      if (part.decoration === 'reverse') {
        let bg = part.bg
        part.bg = part.fg
        part.fg = bg
      }
      if (part.action) part.action(terminal)

      let style = (part.fg & 0xFF) + ((part.bg & 0xFF) << 8) +
        ((attrs & 0xFF) << 16)
      terminal.write(new io.FormattedString([part.content, style]))
    }
  }
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
