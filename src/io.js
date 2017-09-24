const EventEmitter = require('events')

class ANSIParser {
  constructor (handler) {
    this.reset()
    this.handler = handler
    this.joinChunks = true
  }
  reset () {
    this.currentSequence = 0
    this.sequence = ''
  }
  parseSequence (sequence) {
    if (sequence[0] === '[') {
      let type = sequence[sequence.length - 1]
      let content = sequence.substring(1, sequence.length - 1)

      let numbers = content ? content.split(';').map(i => +i.replace(/\D/g, '')) : []
      let numOr1 = numbers.length ? numbers[0] : 1
      if (type === 'H' || type === 'f') {
        this.handler('set-cursor', (numbers[0] | 0) - 1, (numbers[1] | 0) - 1)
      } else if (type >= 'A' && type <= 'D') {
        this.handler(`move-cursor-${type <= 'B' ? 'y' : 'x'}`, ((type === 'B' || type === 'C') ? 1 : -1) * numOr1)
      } else if (type === 'E' || type === 'F') {
        this.handler('move-cursor-line', (type === 'E' ? 1 : -1) * numOr1)
      } else if (type === 'G') {
        this.handler('set-cursor-x', numOr1 - 1)
      } else if (type === 'J') {
        let number = numbers.length ? numbers[0] : 0
        if (number === 0) this.handler('clear-screen-after')
        if (number === 1) this.handler('clear-screen-before')
        if (number === 2) this.handler('clear-screen')
      } else if (type === 'K') {
        let number = numbers.length ? numbers[0] : 0
        if (number === 0) this.handler('clear-line-after')
        if (number === 1) this.handler('clear-line-before')
        if (number === 2) this.handler('clear-line')
      } else if (type === 'L') {
        this.handler('insert-lines', numOr1)
      } else if (type === 'M') {
        this.handler('delete-lines', numOr1)
      } else if (type === 'P') {
        this.handler('delete', numOr1)
      } else if (type === 'S') {
        this.handler('scroll-up')
      } else if (type === 'T') {
        this.handler('scroll-down')
      } else if (type === '@') {
        this.handler('insert-blanks', numOr1)
      } else if (type === 'q') this.handler('set-cursor-style', numOr1)
      else if (type === 's') this.handler('save-cursor')
      else if (type === 'u') this.handler('restore-cursor')
      else if (type === 'm') {
        if (!numbers.length) {
          this.handler('reset-style')
          return
        }
        while (numbers.length) {
          let type = numbers.shift()
          if (type === 0) this.handler('reset-style')
          if (type === 1) this.handler('add-attrs', 1) // bold
          else if (type === 2) this.handler('add-attrs', 1 << 1) // faint
          else if (type === 3) this.handler('add-attrs', 1 << 2) // italic
          else if (type === 4) this.handler('add-attrs', 1 << 3) // underline
          else if (type === 5 || type === 6) this.handler('add-attrs', 1 << 4) // blink
          else if (type === 7) this.handler('add-attrs', -1) // invert
          else if (type === 9) this.handler('add-attrs', 1 << 6) // strike
          else if (type === 20) this.handler('add-attrs', 1 << 5) // fraktur
          else if (type === 21) this.handler('remove-attrs', 1) // remove bold
          else if (type === 22) this.handler('remove-attrs', 0b11) // remove bold & faint
          else if (type === 23) this.handler('remove-attrs', 0b100100) // remove italic & fraktur
          else if (type === 24) this.handler('remove-attrs', 0b01000) // remove underline
          else if (type === 25) this.handler('remove-attrs', 0b10000) // remove blink
          else if (type === 27) this.handler('remove-attrs', -1) // remove inverse
          else if (type >= 30 && type <= 37) this.handler('set-color-fg', type % 10)
          else if (type >= 40 && type <= 47) this.handler('set-color-bg', type % 10)
          else if (type === 39) this.handler('reset-color-fg')
          else if (type === 49) this.handler('reset-color-bg')
          else if (type >= 90 && type <= 98) this.handler('set-color-fg', (type % 10) + 8)
          else if (type >= 100 && type <= 108) this.handler('set-color-bg', (type % 10) + 8)
          else if (type === 38 || type === 48) {
            if (numbers.shift() === 5) {
              let color = (numbers.shift() | 0) & 0xFF
              if (type === 38) this.handler('set-color-fg', color)
              if (type === 48) this.handler('set-color-bg', color)
            }
          }
        }
      } else if (type === 'h' || type === 'l') {
        if (content.startsWith('?')) {
          let mode = +content.substr(1)
          if (mode === 25) {
            if (type === 'h') this.handler('show-cursor')
            else if (type === 'l') this.handler('hide-cursor')
          } else if (mode >= 1047 && mode <= 1049) {
            if (mode === 1047 || mode === 1049) {
              if (type === 'h') {
                this.handler('save-cursor')
                if (mode === 1049) this.handler('set-cursor', 0, 0)
              } else this.handler('restore-cursor')
            }
            if (mode === 1048 || mode === 1049) {
              if (type === 'h') this.handler('enable-alternate-buffer')
              else this.handler('disable-alternate-buffer')
            }
          }
        }
      }
    } else if (sequence[0] === ']') {
      // OSC
      sequence = sequence.substr(1)
      let data = sequence.split(';')
      let type = +data.shift()
      if (type === 0) {
        this.handler('window-title', data[0])
      }
    } else if (sequence[0] === '(' || sequence[0] === ')') {
      // designate character set
      // do nothing.
    }
  }
  write (text) {
    for (let character of text.toString()) {
      let code = character.codePointAt(0)
      if (code === 0x1b && !this.currentSequence) this.currentSequence = 1
      else if (code === 0x9d && !this.currentSequence) this.currentSequence = 3
      else if (this.currentSequence === 1 && character === '[') {
        this.currentSequence = 2
        this.sequence += '['
      } else if (this.currentSequence === 1 && character === ']') {
        this.currentSequence = 3
        this.sequence += ']'
      } else if (this.currentSequence === 1 && (character === '(' || character === ')')) {
        this.currentSequence = 4
        this.sequence += character
      } else if (this.currentSequence > 1 &&
          (code === 0x1b || code === 0x9c || code === 0x07)) {
        // might be a string terminator (ESC \, \x9C, or BEL), here's a hack
        this.parseSequence(this.sequence)
        this.currentSequence = code === 0x1b ? 1 : 0
        this.sequence = ''
      } else if (this.currentSequence === 2 && character.match(/[\x40-\x7e]/)) {
        this.parseSequence(this.sequence + character)
        this.currentSequence = 0
        this.sequence = ''
      } else if (this.currentSequence > 1) {
        this.sequence += character
        if (this.currentSequence === 4) {
          this.parseSequence(this.sequence)
          this.currentSequence = 0
          this.sequence = ''
        }
      } else if (this.currentSequence === 1) {
        if (character === '\\') {
          // string terminator
          this.currentSequence = 0
        } else {
          // something something nothing
          this.currentSequence = 0
          this.handler('write', character)
        }
      } else if (code < 0x03) this.handler('_null')
      else if (code === 0x03) this.handler('sigint')
      else if (code <= 0x06) this.handler('_null')
      else if (code === 0x07) this.handler('bell')
      else if (code === 0x08) this.handler('back')
      else if (code === 0x09) this.handler('tab')
      else if (code === 0x0a) this.handler('new-line')
      else if (code === 0x0d) this.handler('return')
      else if (code === 0x15) this.handler('delete-line')
      else if (code === 0x17) this.handler('delete-word')
      else this.handler('write', character)
    }
    if (!this.joinChunks) this.reset()
  }
}
const TERM_DEFAULT_STYLE = 0
const TERM_MIN_DRAW_DELAY = 10

let getRainbowColor = t => {
  let r = Math.floor(Math.sin(t) * 2.5 + 2.5)
  let g = Math.floor(Math.sin(t + 2 / 3 * Math.PI) * 2.5 + 2.5)
  let b = Math.floor(Math.sin(t + 4 / 3 * Math.PI) * 2.5 + 2.5)
  return 16 + 36 * r + 6 * g + b
}

class ScrollingTerminal extends EventEmitter {
  constructor (width, height) {
    super()

    this.width = width
    this.height = height
    this.parser = new ANSIParser((...args) => this.handleParsed(...args))

    this.reset()

    this._lastLoad = 0
  }
  reset () {
    this.style = TERM_DEFAULT_STYLE
    this.cursor = { x: 0, y: 0, style: 1, visible: true }
    this.savedCursor = { x: 0, y: 0 }
    this.trackMouse = false
    this.rainbow = false
    this.alternateBufferEnabled = false
    this.alternateBuffer = []
    this.parser.reset()
    this.clear()
  }
  isCursorHanging () {
    return this.cursor.x === this.width
  }
  setAlternateBuffer (enabled) {
    if (enabled !== this.alternateBufferEnabled) {
      this.alternateBufferEnabled = enabled

      let [screen, altScreen] = [this.screen, this.alternateBuffer]
      this.alternateBuffer = screen
      this.screen = altScreen

      // clear screen
      this.clear(TERM_DEFAULT_STYLE)
    }

    this.scheduleLoad()
  }
  clear (style) {
    this.screen = []
    for (let y = 0; y < this.height; y++) {
      this.screen.push([])
      this.clearLine(y, style)
    }
  }
  clearLine (n, style) {
    if (n < 0 || n >= this.height) return
    if (style === undefined) style = this.style
    this.screen[n] = []
    for (let x = 0; x < this.width; x++) this.screen[n].push([' ', style])
  }
  generateBlanks (n) {
    let blanks = []
    for (let i = 0; i < n; i++) blanks.push([' ', this.style])
    return blanks
  }
  clearLineBefore (n, x) {
    if (n < 0 || n >= this.height) return
    if (x < 0 || x >= this.width) return
    this.screen[n].splice(0, x, this.generateBlanks(x))
  }
  clearLineAfter (n, x) {
    if (n < 0 || n >= this.height) return
    if (x < 0 || x >= this.width) return
    this.screen[n].splice(x)
    this.screen[n].push(...this.generateBlanks(this.width - x))
  }
  scroll (visual) {
    this.screen.splice(0, 1)
    this.clearLine(this.screen.length, TERM_DEFAULT_STYLE)
    if (!visual) {
      this.cursor.y--
      this.clampCursor()
    }
  }
  scrollDown (visual) {
    this.screen.unshift([])
    this.clearLine(0, TERM_DEFAULT_STYLE)
    this.screen.splice(this.screen.length - 1)
    if (!visual) {
      this.cursor.y++
      this.clampCursor()
    }
  }
  newLine () {
    this.cursor.y++
    if (this.cursor.y >= this.height) this.scroll()
  }
  writeChar (character) {
    if (this.cursor.x >= this.width) {
      this.cursor.x = 0
      this.newLine()
    }
    this.screen[this.cursor.y][this.cursor.x] = [character, this.style]
    this.cursor.x++
  }
  moveBack (n = 1) {
    for (let i = 0; i < n; i++) {
      this.cursor.x--
      if (this.cursor.x < 0) {
        if (this.cursor.y > 0) this.cursor.x = this.width - 1
        else this.cursor.x = 0
        this.cursor.y = Math.max(0, this.cursor.y - 1)
      }
    }
  }
  moveForward (n = 1) {
    for (let i = 0; i < n; i++) {
      this.cursor.x++
      if (this.cursor.x >= this.width) {
        this.cursor.x = 0
        this.cursor.y++
        if (this.cursor.y >= this.height) this.scroll()
      }
    }
  }
  deleteChar () {
    this.moveBack()
    this.screen[this.cursor.y].push([' ', TERM_DEFAULT_STYLE])
    this.screen[this.cursor.y].splice(this.cursor.x, 1)
  }
  deleteForward (n = 1) {
    n = Math.min(this.width - this.cursor.x, n)
    for (let i = 0; i < n; i++) {
      this.screen[this.cursor.y].push([' ', TERM_DEFAULT_STYLE])
    }
    this.screen[this.cursor.y].splice(this.cursor.x, n)
  }
  insertBlanks (n = 1) {
    let line = this.screen[this.cursor.y]
    let before = line.slice(0, this.cursor.x)
    let blanks = this.generateBlanks(n, before[before.length - 1][1])
    let after = line.slice(this.cursor.x)
    this.screen[this.cursor.y] = before.concat(blanks).concat(after)
    this.screen[this.cursor.y].splice(this.width)
    this.scheduleLoad()
  }
  insertLines (n) {
    let lines = []
    for (let i = 0; i < n; i++) {
      lines.push(this.generateBlanks(this.width))
    }
    this.screen.splice(this.cursor.y, 0, ...lines)
    this.screen.splice(this.height) // delete overflow
  }
  deleteLines (n) {
    this.screen.splice(this.cursor.y, n)
    for (let i = this.screen.length; i < this.height; i++) {
      this.screen.push(this.generateBlanks(this.width, TERM_DEFAULT_STYLE))
    }
  }
  clampCursor () {
    if (this.cursor.x < 0) this.cursor.x = 0
    if (this.cursor.y < 0) this.cursor.y = 0
    if (this.cursor.x > this.width) this.cursor.x = this.width // can be dangling
    if (this.cursor.y > this.height - 1) this.cursor.y = this.height - 1
  }
  handleParsed (action, ...args) {
    if (action === 'write') {
      this.writeChar(args[0])
    } else if (action === 'delete') {
      this.deleteForward(args[0])
    } else if (action === 'insert-blanks') {
      this.insertBlanks(args[0])
    } else if (action === 'clear-screen') {
      this.clear()
    } else if (action === 'clear-screen-before') {
      this.clearLineBefore(this.cursor.y, this.cursor.x)
      for (let i = 0; i < this.cursor.y; i++) this.clearLine(i, TERM_DEFAULT_STYLE)
    } else if (action === 'clear-screen-after') {
      this.clearLineAfter(this.cursor.y, this.cursor.x)
      for (let i = this.cursor.y + 1; i < this.height; i++) this.clearLine(i, TERM_DEFAULT_STYLE)
    } else if (action === 'clear-line') {
      this.clearLine(this.cursor.y)
    } else if (action === 'clear-line-before') {
      this.clearLineBefore(this.cursor.y, this.cursor.x)
    } else if (action === 'clear-line-after') {
      this.clearLineAfter(this.cursor.y, this.cursor.x)
    } else if (action === 'scroll-up') {
      this.scroll()
    } else if (action === 'scroll-down') {
      this.scrollDown()
    } else if (action === 'bell') {
      this.emit('bell')
    } else if (action === 'window-title') {
      this.emit('window-title', args[0])
    } else if (action === 'back') {
      this.moveBack()
    } else if (action === 'new-line') {
      this.newLine()
      this.cursor.x = 0
    } else if (action === 'insert-lines') {
      this.insertLines(args[0])
    } else if (action === 'delete-lines') {
      this.deleteLines(args[0])
    } else if (action === 'return') {
      this.cursor.x = 0
    } else if (action === 'set-cursor') {
      this.cursor.x = args[1]
      this.cursor.y = args[0]
      this.clampCursor()
    } else if (action === 'move-cursor-y') {
      this.cursor.y += args[0]
      this.clampCursor()
    } else if (action === 'move-cursor-x') {
      this.cursor.x += args[0]
      this.clampCursor()
    } else if (action === 'move-cursor-line') {
      this.cursor.x = 0
      this.cursor.y += args[0]
      this.clampCursor()
    } else if (action === 'set-cursor-x') {
      this.cursor.x = args[0]
    } else if (action === 'set-cursor-style') {
      this.cursor.style = Math.max(0, Math.min(6, args[0]))
    } else if (action === 'save-cursor') {
      this.savedCursor.x = this.cursor.x
      this.savedCursor.y = this.cursor.y
    } else if (action === 'restore-cursor') {
      this.cursor.x = this.savedCursor.x
      this.cursor.y = this.savedCursor.y
    } else if (action === 'reset-style') {
      this.style = TERM_DEFAULT_STYLE
    } else if (action === 'add-attrs') {
      this.style |= (args[0] << 16)
    } else if (action === 'remove-attrs') {
      this.style &= ~(args[0] << 16)
    } else if (action === 'set-color-fg') {
      this.style = (this.style & 0xFFFFFF00) | (1 << 8 << 16) | args[0]
    } else if (action === 'set-color-bg') {
      this.style = (this.style & 0xFFFF00FF) | (1 << 9 << 16) | (args[0] << 8)
    } else if (action === 'reset-color-fg') {
      this.style = this.style & 0xFFFEFF00
    } else if (action === 'reset-color-bg') {
      this.style = this.style & 0xFFFD00FF
    } else if (action === 'hide-cursor') {
      this.cursor.visible = false
    } else if (action === 'show-cursor') {
      this.cursor.visible = true
    }
  }
  write (text) {
    this.parser.write(text)
    this.scheduleLoad()
  }
  serialize () {
    let serialized = 'S'
    serialized += String.fromCodePoint(this.height + 1) + String.fromCodePoint(this.width + 1)
    serialized += String.fromCodePoint(this.cursor.y + 1) + String.fromCodePoint(this.cursor.x + 1)

    let attributes = +this.cursor.visible
    attributes |= (3 << 5) * +this.trackMouse // track mouse controls both
    attributes |= 3 << 7 // buttons/links always visible
    attributes |= (this.cursor.style << 9)
    serialized += String.fromCodePoint(attributes + 1)

    let lastStyle = null
    let index = 0
    for (let cell of this.screen.reduce((a, b) => a.concat(b))) {
      let style = cell[1]
      if (this.rainbow) {
        let x = index % this.width
        let y = Math.floor(index / this.width)
        // C instead of F in mask and 1 << 8 in attrs to change attr bits 8 and 9
        style = (style & 0xFFFC0000) | (1 << 8 << 16) | getRainbowColor((x + y) / 10 + Date.now() / 1000)
        index++
      }
      if (style !== lastStyle) {
        let foreground = style & 0xFF
        let background = (style >> 8) & 0xFF
        let attributes = (style >> 16) & 0xFFFF
        let setForeground = foreground !== (lastStyle & 0xFF)
        let setBackground = background !== ((lastStyle >> 8) & 0xFF)
        let setAttributes = attributes !== ((lastStyle >> 16) & 0xFFFF)

        if (setForeground && setBackground) serialized += '\x03' + String.fromCodePoint((style & 0xFFFF) + 1)
        else if (setForeground) serialized += '\x05' + String.fromCodePoint(foreground + 1)
        else if (setBackground) serialized += '\x06' + String.fromCodePoint(background + 1)
        if (setAttributes) serialized += '\x04' + String.fromCodePoint(attributes + 1)
        lastStyle = style
      }
      serialized += cell[0]
    }
    return serialized
  }
  scheduleLoad () {
    clearTimeout(this._scheduledLoad)
    if (this._lastLoad < Date.now() - TERM_MIN_DRAW_DELAY) {
      this.emit('update')
    } else {
      this._scheduledLoad = setTimeout(() => {
        this.emit('update')
      }, TERM_MIN_DRAW_DELAY - this._lastLoad)
    }
  }
  rainbowTimer () {
    if (!this.rainbow) return
    clearInterval(this._rainbowTimer)
    this._rainbowTimer = setInterval(() => {
      if (this.rainbow) this.scheduleLoad()
    }, 50)
  }
}

module.exports = { ANSIParser, ScrollingTerminal }
