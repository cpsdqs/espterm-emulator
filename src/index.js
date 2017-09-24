const express = require('express')
const WebSocket = require('ws')
const http = require('http')
const path = require('path')
const url = require('url')
const fs = require('fs')

const shell = require('./shell')
const variables = require('./variables')

// path to the ESPTerm repository's html folder
if (!process.argv[2]) {
  console.error('Must pass ESPTerm html folder location as argument')
  process.exit(-1)
}
const base = path.resolve(process.argv[2])
console.log(`Using ${base}`)

const app = express()

let applyTemplate = function (file) {
  file = file.toString()
  file = file.replace(/%[\w:]+%/g, match => {
    let value = variables[match.replace(/%(\w:)?/g, '')]
    if (value === undefined) return match
    return value
  })
  return file
}

let serveTemplate = function (res, path) {
  fs.readFile(`${base}/${path}`, (err, file) => {
    if (err) {
      res.send('Error')
      console.log(err)
    } else {
      res.set('Content-Type', 'text/html')
      res.send(applyTemplate(file))
    }
  })
}

app.get(/.tpl/, (req, res) => serveTemplate(res, req.path))
app.get('/', (req, res) => serveTemplate(res, 'term.tpl'))
app.get('/cfg/term', (req, res) => serveTemplate(res, 'cfg_term.tpl'))
app.get('/cfg/network', (req, res) => serveTemplate(res, 'cfg_network.tpl'))
app.get('/cfg/system', (req, res) => serveTemplate(res, 'cfg_system.tpl'))
app.get('/cfg/wifi', (req, res) => serveTemplate(res, 'cfg_wifi.tpl'))
app.get('/help', (req, res) => serveTemplate(res, 'help.html'))
app.get('/about', (req, res) => serveTemplate(res, 'about.tpl'))
app.get(/^\/cfg\/\w+\/set/, (req, res) => {
  Object.assign(variables, req.query)
  res.redirect(req.path.replace('/set', ''))
})

app.get('/system/ping', (req, res) => {
  res.send('pong')
})

let encode2B = i =>
  String.fromCharCode((i % 127) + 1) +
    String.fromCharCode(Math.floor(i / 127) + 1)

// from MightyPork/espterm
let encode3B = n => {
  var lsb, msb, xsb
  lsb = (n % 127)
  n = (n - lsb) / 127
  lsb += 1
  msb = (n % 127)
  n = (n - msb) / 127
  msb += 1
  xsb = (n + 1)
  return String.fromCharCode(lsb) + String.fromCharCode(msb) +
    String.fromCharCode(xsb)
}

let decode2B = (str, i) =>
  str.charCodeAt(i) - 1 + (str.charCodeAt(i + 1) - 1) * 127

let getUpdateString = function () {
  let str = 'S'
  str += String.fromCodePoint(shell.options.height + 1)
  str += String.fromCodePoint(shell.options.width + 1)
  str += String.fromCodePoint(shell.terminal.cursorPos[1] + 1)
  str += String.fromCodePoint(shell.terminal.cursorPos[0] + 1)
  let attributes = 0
  attributes |= 0b1 * +shell.terminal.state.cursorVisible
  attributes |= 0b10 * +shell.options.cursorHanging
  attributes |= 0b100 * +shell.options.cursorKeysAppMode
  attributes |= 0b1000 * +shell.options.numpadKeysAppMode
  attributes |= 0b10000 * +shell.options.functionKeysMode
  attributes |= 0b100000 * +(shell.options.trackMouseClicks ||
    shell.terminal.state.alternateBuffer)
  attributes |= 0b1000000 * +shell.options.trackMouseMovement
  attributes |= 0b10000000 * +shell.options.enableButtons
  attributes |= 0b100000000 * +shell.options.enableMenu
  attributes |= shell.terminal.state.cursorStyle << 9
  str += String.fromCodePoint(attributes + 1)

  // TODO: add compression
  str += shell.terminal.render()
  return str
}

let title = 'ESPTerm'
shell.on('update-title', newTitle => { title = newTitle })

let getTitleString = function () {
  let buttons = []
  for (let i = 1; i <= 5; i++) buttons.push(variables[`btn${i}`])
  return `T${title}` + buttons.map(name => '\x01' + name).join('')
}

app.get('/term/init', (req, res) => {
  res.send(getUpdateString())
})

app.use(express.static(base))

const server = http.createServer(app)
server.listen(3000, () => console.log('Listening on :3000'))

const ws = new WebSocket.Server({ server: server })

let hasConnection = false

ws.on('connection', (ws, request) => {
  if (hasConnection) {
    ws.close()
    return
  }
  hasConnection = true

  const ip = request.connection.remoteAddress
  console.log('connected from ' + ip)

  ws.send(getTitleString())
  ws.send(getUpdateString())

  let update = () => ws.send(getUpdateString())
  let updateTitle = () => ws.send(getTitleString())
  let emitBell = () => ws.send('B')

  shell.terminal.on('update', update)
  shell.on('update-title', updateTitle)
  shell.on('bell', emitBell)

  let heartbeat = setInterval(() => {
    ws.send('.')
  }, 1000)

  ws.on('message', message => {
    let type = message[0]
    let content = message.slice(1)

    if (type === 's') {
      // string input
      shell.write(content)
    } else if (type === 'b') {
      let button = content.charCodeAt(0)
      // what to do?
    } else if (type === 'm' || type === 'p' || type === 'r') {
      let row = decode2B(content, 0)
      let column = decode2B(content, 2)
      let button = decode2B(content, 4)
      let modifiers = decode2B(content, 6)

      if (shell.terminal.state.alternateBuffer && 4 <= button && button <= 5) {
        if (button === 4) {
          shell.write('\x1bOA')
        } else if (button === 5) {
          shell.write('\x1bOB')
        }
      }
    }
  })

  ws.on('close', () => {
    clearInterval(heartbeat)
    shell.terminal.removeListener('update', update)
    shell.removeListener('update-title', updateTitle)
    shell.removeListener('bell', emitBell)
    console.log('disconnected')
    hasConnection = false
  })
})
