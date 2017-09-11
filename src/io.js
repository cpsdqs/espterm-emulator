const EventEmitter = require('events')
let io = module.exports = {}

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

// string with format
// style property: LSB to MSB
// 4 foreground 0-16
// 4 background 0-16
// 1 bold
// 1 faint
// 1 italic
// 1 underline
// 1 blink
// 1 fraktur
// 1 strike
// 1 reverse
io.FormattedString = class FormattedString {
  constructor (...args) {
    this.parts = []
    this.length = 0

    for (let part of args) {
      if (typeof part === 'string') {
        this.parts.push([part, 7])
        this.length += part.length
      } else if (Array.isArray(part)) {
        this.parts.push(['' + part[0], part[1]])
        this.length += ('' + part[0]).length
      }
    }
  }
  updateLength () {
    this.length = 0
    for (let item of this.parts) this.length += item[0].length
  }
  setStyle (part, style) {
    this.parts[part][1] = style & 0xFFFFFF
  }
  setContent (part, content) {
    this.parts[part][0] = '' + content
    this.updateLength()
  }
  addPart (content, style) {
    this.parts.push(['' + content, +style & 0xFFFFFF])
    this.length += content.length
  }
  partIndexOf (index) {
    if (index > this.length - 1) return null
    let idx = 0
    for (let pi in this.parts) {
      let part = this.parts[pi]
      let pidx = idx
      idx += part[0].length
      if (idx >= index) {
        return [+pi, index - pidx]
      }
    }
  }
  toPlainText () {
    let str = ''
    for (let part of this.parts) {
      str += part[0]
    }
    return str
  }

  get lastStyle () {
    return this.parts[this.parts.length - 1][1]
  }

  slice (start, end) {
    let pos = 0
    let slicePart = null
    let sliceParts = []
    if (start === undefined) start = 0
    if (end === undefined) end = Infinity

    for (let part of this.parts) {
      for (let character of part[0]) {
        if (pos < start) {
          pos++
          continue
        }
        if (pos >= end) break

        if (!slicePart || part[1] !== slicePart[1]) {
          sliceParts.push(slicePart = ['', part[1]])
        }
        slicePart[0] += character
        pos++
      }
    }

    if (!sliceParts.length) sliceParts.push('')
    return new io.FormattedString(...sliceParts)
  }

  concat (other) {
    return new io.FormattedString(...this.parts.concat(other.parts))
  }

  // this overwrites this FormattedString at a given index with another one
  // if append is true it allows for overflow behind
  overwrite (index, fstring, append) {
    let pos = index
    // space is a variable used later on
    let space = true
    if (pos === undefined) return this
    if (!fstring.length) return this
    for (let part of fstring.parts) {
      for (let char of part[0]) {
        let idx = this.partIndexOf(pos)
        // the index is outside the string length
        if (!idx && append) {
          // only add spaces in between if append is true and space is true
          if (pos > this.length && space) {
            this.parts.push([new Array(pos - this.length + 1).join(' '), 7])
            space = false
          }
          this.parts.push([char, part[1]])
          continue
        } else if (!idx) continue

        // the part at the target index
        let pt = this.parts[idx[0]]
        // does the part match the wanted styles?
        if (pt[1] !== part[1]) {
          // nope.
          // determine if the character goes inside the part or just behind it
          let idxi = idx[1] === pt[0].length
          // split the part in two, right where the character goes
          let pt2 = pt[0].substr(idx[1] + 1)
          pt[0] = pt[0].substr(0, idx[1])
          // if the second part isn't empty, put it behind the original part
          if (pt2 !== '') {
            this.parts.splice(idx[0] + 1, 0, [pt2, pt[1]])
          } else if (pt2 === '' && idxi) {
            // if it's empty, find the next non-empty part and take the first character
            // but only if the character goes directly after the part, not inside the part
            let counter = 0
            while (this.parts[idx[0] + (++counter)] &&
                this.parts[idx[0] + counter][0] === '') {
              // already counting above, nothing to do here
            }
            if (this.parts[idx[0] + counter]) {
              let rmp = this.parts[idx[0] + counter]
              rmp[0] = rmp[0].substr(1)
              if (rmp[0] === '') {
                this.parts.splice(idx[0] + counter, 1)
              }
            }
          }
          // create a new part and put the character in it
          this.parts.splice(idx[0] + 1, 0, [char, part[1]])
        } else {
          // it matches the styles. The character will now be put in this part
          // and won't create a new part

          // check if the character goes right behind the part
          if (idx[1] >= pt[0].length) {
            // if it does, find the next non-empty part and take the first character
            let counter = 0
            while (this.parts[idx[0] + (++counter)] &&
              this.parts[idx[0] + counter][0] === '') {
              // already counting above, nothing to do here
            }
            if (this.parts[idx[0] + counter]) {
              let rmp = this.parts[idx[0] + counter]
              rmp[0] = rmp[0].substr(1)
              if (rmp[0] === '') {
                this.parts.splice(idx[0] + counter, 1)
              }
            }
          }
          // put the character in the part
          pt[0] = pt[0].substr(0, idx[1]) + char + pt[0].substr(idx[1] + 1)
        }
        pos++
      }
    }
    this.updateLength()
    return this
  }
}

io.Terminal = class Terminal extends EventEmitter {
  constructor (width, height) {
    super()

    this.width = width
    this.height = height

    this.lines = []

    // alternate buffer
    this.altLines = []

    for (let i = 0; i < this.height; i++) {
      this.lines.push(new io.FormattedString(''))
      this.altLines.push(new io.FormattedString(''))
    }


    this.cursorPos = [0, 0]
    this.state = {
      cursorVisible: true,
      cursorStyle: 1,
      alternateBuffer: false,
      savedCursorPos: null
    }
  }

  setAlternateBuffer (enabled) {
    if (enabled !== this.state.alternateBuffer) {
      this.state.alternateBuffer = enabled

      // clear alternate buffer
      if (!this.state.alternateBuffer) this.clear()

      let altLines = this.altLines
      this.altLines = this.lines
      this.lines = altLines
    }
    this.draw()
  }

  clear () {
    this.lines = []
    for (let i = 0; i < this.height; i++) {
      this.lines.push(new io.FormattedString(''))
    }
    this.draw()
  }
  clearLine (n) {
    let line = this.lines[n]
    if (!line) return
    line = new io.FormattedString('')
  }
  clearLinePre () {
    let line = this.lines[this.cursorPos[1]]
    if (!line) {
      line = this.lines[this.cursorPos[1]] = new io.FormattedString('')
    }
    line.overwrite(0, new io.FormattedString(' '.repeat(this.cursorPos[0]), 7))
  }
  clearLineRest () {
    let line = this.lines[this.cursorPos[1]]
    if (!line) {
      line = this.lines[this.cursorPos[1]] = new io.FormattedString('')
    }
    let len = this.width - this.cursorPos[0]
    line.overwrite(this.cursorPos[0],
      new io.FormattedString(' '.repeat(len), 7))
  }
  insertBlank (n) {
    let line = this.lines[this.cursorPos[1]]
    let before = line.slice(0, this.cursorPos[0])
    let blanks = new io.FormattedString([' '.repeat(n), before.lastStyle])
    let after = line.slice(this.cursorPos[0])
    this.lines[this.cursorPos[1]] = before.concat(blanks).concat(after)
    this.draw()
  }
  deleteForward (n) {
    let line = this.lines[this.cursorPos[1]]
    let before = line.slice(0, this.cursorPos[0])
    let after = line.slice(this.cursorPos[0] + n)
    this.lines[this.cursorPos[1]] = before.concat(after)
    this.draw()
  }
  insertLines (n) {
    let lines = []
    for (let i = 0; i < n; i++) lines.push(new io.FormattedString(''))
    this.lines.splice(this.cursorPos[1], 0, ...lines)
    this.lines.splice(this.height) // delete overflow
  }
  deleteLines (n) {
    this.lines.splice(this.cursorPos[1], n)
    for (let i = this.lines.length; i < this.height; i++) {
      this.lines.push(new io.FormattedString(''))
    }
  }

  write (formatted) {
    // (over-)writes formatted text at cursor position and draws
    for (let part of formatted.parts) {
      let line = this.lines[this.cursorPos[1]]

      if (!line) {
        line = this.lines[this.cursorPos[1]] = new io.FormattedString('')
      }

      let partString = part[0]
      let subparts = []
      let firstLine = true

      while (partString) {
        let splitIndex = this.width
        if (firstLine) splitIndex -= this.cursorPos[0]

        // break at newline
        let indexOfNewline = partString.indexOf('\n')
        if (indexOfNewline > -1 && indexOfNewline < splitIndex) {
          splitIndex = indexOfNewline + 1
        }

        subparts.push(partString.substr(0, splitIndex))
        partString = partString.substr(splitIndex)
        firstLine = false
      }

      for (let subpart of subparts) {
        line.overwrite(this.cursorPos[0],
          new io.FormattedString([subpart, part[1]]), true)
        this.cursorPos[0] += subpart.length
        if (this.cursorPos[0] >= this.width) {
          this.cursorPos[0] = 0
          this.cursorPos[1]++
          if (this.cursorPos[1] > this.height - 1) this.scroll()
          line = this.lines[this.cursorPos[1]]
          if (!line) {
            line = this.lines[this.cursorPos[1]] = new io.FormattedString('')
          }
        }
      }
    }

    this.draw()
  }
  scroll (visual) {
    this.lines.splice(0, 1)
    if (!visual) {
      this.cursorPos[1]--
      this.clampCursorPos()
    }
  }
  scrollDown (visual) {
    this.terminal.lines.unshift(new io.FormattedString())
    this.terminal.lines.splice(this.terminal.lines.length - 1)
    if (!visual) {
      this.cursorPos[1]++
      this.clampCursorPos()
    }
  }
  draw () {
    this.emit('update')
  }

  clampCursorPos () {
    if (this.cursorPos[0] < 0) this.cursorPos[0] = 0
    if (this.cursorPos[1] < 0) this.cursorPos[1] = 0
    if (this.cursorPos[0] > this.width - 1) this.cursorPos[0] = this.width - 1
    if (this.cursorPos[1] > this.height - 1) this.cursorPos[1] = this.height - 1
  }

  render () {
    let styleToString = style => {
      let fg = style & 0xFF
      let bg = style >> 8 & 0xFF
      let attrs = style >> 16 & 0xFF
      let col = encode3B(fg + (bg << 8))
      return '\x03' + col + '\x04' + encode2B(attrs)
    }

    return this.lines.map(line => {
      let additional = ' '.repeat(Math.max(0, this.width - line.length))
      additional = styleToString(7) + additional
      return line.parts.map(part =>
        styleToString(part[1]) + part[0]).join('') + additional
    }).join('')
  }
}
