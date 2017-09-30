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

let applyTemplate = function (file, path = '') {
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
      res.send(applyTemplate(file, path))
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
  if ('default_fg' in req.query || 'default_bg' in req.query || 'theme' in req.query) {
    // hack
    shell.emit('update-theme')
  }
})
app.get('/cfg/wifi/scan', (req, res) => {
  res.send(JSON.stringify({
    result: {
      inProgress: 0,
      "APs": [
        {
          essid: `PID: ${process.pid}`,
          rssi_perc: 100,
          enc: 0
        }
      ]
    }
  }))
})

app.get('/system/ping', (req, res) => {
  res.send('pong')
})

let decode2B = (str, i) =>
  str.charCodeAt(i) - 1 + (str.charCodeAt(i + 1) - 1) * 127

let encodeAsCodePoint = i => String.fromCodePoint(+i + 1)

const topics = {
  changeScreenOpts: 1,
  changeContentAll: 1 << 1,
  changeContentPart: 1 << 2,
  changeTitle: 1 << 3,
  changeButtons: 1 << 4,
  changeCursor: 1 << 5,
  internal: 1 << 6,
  bell: 1 << 7
}

let getAttributes = () => shell.terminal.getAttributes()
let getStateID = () => shell.terminal.getStateID()
let getTitle = () => shell.terminal.getTitle()
let getBellID = () => shell.terminal.getBellID()
let getScreen = () => shell.terminal.serializeScreen(Date.now() / 1000)
let getCursor = () => shell.terminal.getCursor()

let getTitleStringFor = function (title) {
  let buttons = []
  for (let i = 1; i <= 5; i++) buttons.push(variables[`btn${i}`])
  return `T${title}` + buttons.map(name => '\x01' + name).join('')
}

app.get('/term/init', (req, res) => {
  res.send(getUpdateString())
})

app.use(express.static(base))

const server = http.createServer(app)
let port = 3000
if (Number.isFinite(+process.argv[3])) port = process.argv[3] | 0
server.listen(port, () => console.log(`Listening on :${port}`))

const ws = new WebSocket.Server({ server })

let connections = 0

ws.on('connection', (ws, request) => {
  if (connections >= 2) {
    ws.close()
    return
  }
  connections++

  let trySend = data => {
    if (ws.readyState === 1) ws.send(data)
  }

  const ip = request.connection.remoteAddress
  console.log(`connected from ${ip} (${connections} connection${connections === 1 ? '' : 's'})`)

  let lastAttributes = null
  let lastStateID = null
  let lastScreen = null
  let lastBellID = getBellID()
  let lastTitle = null
  let lastCursor = null
  let sentButtons = false

  let clearAttributes = () => lastAttributes = null

  let update = () => {
    let data = 'U'

    let topicFlags = 0
    let topicData = []

    // check what changed
    let attributes = getAttributes()
    let stateID = getStateID() // tracks screen updates
    let bellID = getBellID()
    let title = getTitle()
    let cursor = getCursor()

    if (attributes !== lastAttributes) {
      lastAttributes = attributes

      topicFlags |= topics.changeScreenOpts
      let data = 'O'
      data += encodeAsCodePoint(shell.terminal.height())
      data += encodeAsCodePoint(shell.terminal.width())
      data += encodeAsCodePoint(variables.theme)
      let defaultFG = variables.default_fg
      let defaultBG = variables.default_bg
      if (defaultFG.toString().match(/^#[\da-f]{6}$/)) defaultFG = parseInt(defaultFG.substr(1), 16) + 256
      if (defaultBG.toString().match(/^#[\da-f]{6}$/)) defaultBG = parseInt(defaultBG.substr(1), 16) + 256
      data += encodeAsCodePoint(defaultFG & 0xFFFF)
      data += encodeAsCodePoint((defaultFG >> 16))
      data += encodeAsCodePoint(defaultBG & 0xFFFF)
      data += encodeAsCodePoint((defaultBG >> 16))
      data += encodeAsCodePoint(attributes)
      topicData.push(data)
    }
    if (title !== lastTitle) {
      lastTitle = title
      topicFlags |= topics.changeTitle
      topicData.push(`T${title}\x01`)
    }
    if (!sentButtons) {
      sentButtons = true
      topicFlags |= topics.changeButtons
      topicData.push(`B${encodeAsCodePoint(5)}1\x012\x013\x014\x015\x01`)
    }
    if (bellID !== lastBellID) {
      lastBellID = bellID
      topicFlags |= topics.bell
      topicData.push('!')
    }
    if (cursor !== lastCursor) {
      lastCursor = cursor
      topicFlags |= topics.changeCursor
      topicData.push(`C${cursor}`)
    }
    if (stateID !== lastStateID) {
      lastStateID = stateID

      let screen = getScreen()
      if (screen !== lastScreen) {
        lastScreen = screen
        topicFlags |= topics.changeContentAll
        topicData.push(screen)
      }
    }

    if (!topicFlags) return

    data += encodeAsCodePoint(topicFlags)
    data += topicData.join('')

    trySend(data)
  }

  let updateInterval = setInterval(update, 30);

  let heartbeat = setInterval(() => {
    trySend('.')
  }, 1000)

  shell.on('update-theme', clearAttributes)

  ws.on('message', message => {
    let type = message[0]
    let content = message.slice(1)

    if (type === 's') {
      // string input
      shell.write(content)
    } else if (type === 'b') {
      let button = content.charCodeAt(0)
      shell.write(String.fromCharCode(button))
    } else if (type === 'm' || type === 'p' || type === 'r') {
      let row = decode2B(content, 0)
      let column = decode2B(content, 2)
      let button = decode2B(content, 4)
      let modifiers = decode2B(content, 6)

      /* if (shell.terminal.state.alternateBuffer && 4 <= button && button <= 5) {
        if (button === 4) {
          shell.write('\x1bOA')
        } else if (button === 5) {
          shell.write('\x1bOB')
        }
      } */
    }
  })

  ws.on('close', () => {
    clearInterval(heartbeat)
    clearInterval(updateInterval)
    shell.removeListener('update-theme', clearAttributes)
    connections--
    console.log(`disconnected ${ip} (${connections} connection${connections === 1 ? '' : 's'} left)`)
  })
})
