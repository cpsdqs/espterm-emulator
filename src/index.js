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

let getStateID = () => shell.terminal.getStateID()
let getTitle = () => shell.terminal.getTitle()
let getBellID = () => shell.terminal.getBellID()

let getUpdateString = function () {
  return shell.terminal.serialize(Date.now() / 1000)
}

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
  if (connections >= 1) {
    ws.close()
    return
  }
  connections++

  let trySend = data => {
    if (ws.readyState === 1) ws.send(data)
  }

  const ip = request.connection.remoteAddress
  console.log(`connected from ${ip} (${connections} connection${connections === 1 ? '' : 's'})`)

  let updateTitle = title => trySend(getTitleStringFor(title))
  let emitBell = () => trySend('B')

  let lastStateID = null
  let lastBellID = getBellID()
  let lastTitle = null

  let update = () => {
    let stateID = getStateID()
    if (stateID !== lastStateID) {
      trySend(getUpdateString())
      lastStateID = stateID
    }
    let bellID = getBellID()
    if (bellID !== lastBellID) {
      emitBell()
      lastBellID = bellID
    }
    let title = getTitle()
    if (lastTitle !== title) {
      updateTitle(title)
      lastTitle = title
    }
  }

  let updateInterval = setInterval(update, 30);

  let heartbeat = setInterval(() => {
    trySend('.')
  }, 1000)

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
    connections--
    console.log(`disconnected ${ip} (${connections} connection${connections === 1 ? '' : 's'} left)`)
  })
})
