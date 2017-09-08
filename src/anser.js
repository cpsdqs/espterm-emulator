// this is modified from https://npm.im/anser version 1.4.2

const io = require('./io')

// This file was originally written by @drudru (https://github.com/drudru/ansi_up), MIT, 2011

class Anser {
  constructor () {
    this.fg = this.bg = this.fg_truecolor = this.bg_truecolor = null
    this.bright = 0
  }

  setupPalette () {
    this.PALETTE_COLORS = []
    for (let i = 0; i < 256; i++) this.PALETTE_COLORS.push(i)
  }

  ansiToJson (txt, options) {
    options = options || {}
    options.json = true
    options.clearLine = false
    return this.process(txt, options, true)
  }

  process (txt, options, markup) {
    let self = this
    let raw_text_chunks = txt.split(/\033\[/)
    let first_chunk = raw_text_chunks.shift() // the first chunk is not the result of the split

    if (options === undefined || options === null) {
      options = {}
    }
    options.clearLine = /\r/.test(txt) // check for Carriage Return
    let color_chunks = raw_text_chunks.map(chunk => this.processChunk(chunk, options, markup))

    if (options && options.json) {
      let first = self.processChunkJson("")
      first.content = first_chunk
      first.clearLine = options.clearLine
      color_chunks.unshift(first)
      if (options.remove_empty) {
        color_chunks = color_chunks.filter(c => !c.isEmpty())
      }
      return color_chunks
    } else {
      color_chunks.unshift(first_chunk)
    }

    return color_chunks.join("")
  }

  processChunkJson (text, options, markup) {
    options = typeof options == "undefined" ? {} : options

    let result = {
      content: text,
      fg: this.fg,
      bg: this.bg,
      fg_truecolor: null,
      bg_truecolor: null,
      clearLine: options.clearLine,
      decoration: null,
      was_processed: false,
      isEmpty: () => !result.content,
      action: null
    }

    // Each "chunk" is the text after the CSI (ESC + "[") and before the next CSI/EOF.
    //
    // This regex matches four groups within a chunk.
    //
    // The first and third groups match code type.
    // We supported only SGR command. It has empty first group and "m" in third.
    //
    // The second group matches all of the number+semicolon command sequences
    // before the "m" (or other trailing) character.
    // These are the graphics or SGR commands.
    //
    // The last group is the text (including newlines) that is colored by
    // the other group"s commands.
    let matches = text.match(/^([!\x3c-\x3f]*)([\d;]*)([\x20-\x2c]*[\x40-\x7e])([\s\S]*)/m)

    if (!matches) return result

    let orig_txt = result.content = matches[4]
    let nums = matches[2].split(";")
    if (!matches[2]) nums = []

    let type = matches[3]
    if (type === 'H' || type === 'f') {
      // move cursor
      result.action = terminal => {
        terminal.cursorPos[0] = (nums[1] | 0) - 1
        terminal.cursorPos[1] = (nums[0] | 0) - 1
        terminal.clampCursorPos()
      }
    } else if (type === 'A') {
      // cursor up
      result.action = terminal => {
        terminal.cursorPos[1] -= nums.length ? (nums[0] | 0) : 1
        terminal.clampCursorPos()
      }
    } else if (type === 'B') {
      // cursor down
      result.action = terminal => {
        terminal.cursorPos[1] += nums.length ? (nums[0] | 0) : 1
        terminal.clampCursorPos()
      }
    } else if (type === 'C') {
      // cursor forward
      result.action = terminal => {
        terminal.cursorPos[0] += nums.length ? (nums[0] | 0) : 1
        terminal.clampCursorPos()
      }
    } else if (type === 'D') {
      // cursor backward
      result.action = terminal => {
        terminal.cursorPos[0] -= nums.length ? (nums[0] | 0) : 1
        terminal.clampCursorPos()
      }
    } else if (type === 'E') {
      // move to beginning of next line
      result.action = terminal => {
        terminal.cursorPos[1] += nums.length ? nums[0] : 1
        terminal.cursorPos[0] = 0
        terminal.clampCursorPos()
      }
    } else if (type === 'F') {
      // move to beginning of previous line
      result.action = terminal => {
        terminal.cursorPos[1] -= nums.length ? nums[0] : 1
        terminal.cursorPos[0] = 0
        terminal.clampCursorPos()
      }
    } else if (type === 'G') {
      // move to n columns
      result.action = terminal => {
        terminal.cursorPos[0] = nums.length ? nums[0] - 1 : 0
      }
    } else if (type === 's') {
      // save cursor pos
      result.action = terminal => {
        this._savedCursorPos = [...terminal.cursorPos]
      }
    } else if (type === 'u') {
      // restore cursor pos
      result.action = terminal => {
        if (this._savedCursorPos) {
          terminal.cursorPos = this._savedCursorPos
        }
      }
    } else if (type === 'J') {
      // clear screen
      let num = nums.length ? (nums[0] | 0) : 0
      if (num === 0) {
        // clear after cursor
        result.action = terminal => {
          terminal.clearLineRest()
          for (let i = terminal.cursorPos[1]; i < terminal.height; i++) {
            terminal.clearLine(i)
          }
        }
      } else if (num === 1) {
        // clear before cursor
        result.action = terminal => {
          for (let i = 0; i < terminal.cursorPos[1]; i++) {
            terminal.clearLine(i)
          }
          terminal.clearLinePre()
        }
      } else if (num === 2) {
        // clear all
        result.action = terminal => {
          terminal.cursorPos = [0, 0]
          terminal.clear()
        }
      }
    } else if (type === 'K') {
      // clear line
      let num = nums.length ? (nums[0] | 0) : 0
      if (num === 0) {
          // clear after cursor
          result.action = terminal =>
            terminal.clearLineRest()
      } else if (num === 1) {
        // clear before cursor
        result.action = terminal =>
          terminal.clearLinePre()
      } else if (num === 2) {
        // clear all
        result.action = terminal =>
          terminal.clearLine(terminal.cursorPos[1])
      }
  } else if (type === 'S') {
    // scroll up
    result.action = terminal => {
      let num = nums.length ? (nums[0] | 0) : 1
      for (let i = 0; i < num; i++) terminal.scroll(true)
    }
  } else if (type === 'T') {
    // scroll down
    result.action = terminal => {
      let num = nums.length ? (nums[0] | 0) : 1
      for (let i = 0; i < num; i++) {
        terminal.scrollDown(true)
      }
    }
  } else if (type === 'P') {
    // delete
    result.action = terminal => {
      let num = nums.length ? (nums[0] | 0) : 1
      terminal.deleteForward(num)
    }
  } else if (type === '@') {
    // insert blank characters
    result.action = terminal => {
      let num = nums.length ? (nums[0] | 0) : 1
      terminal.insertBlank(num)
    }
  } else if (type === 'L') {
    // insert lines
    result.action = terminal => {
      let num = nums.length ? (nums[0] | 0) : 1
      terminal.insertLines(num)
    }
  } else if (type === 'M') {
    // delete lines
    result.action = terminal => {
      let num = nums.length ? (nums[0] | 0) : 1
      terminal.deleteLines(num)
    }
  } else if (type === 'h' || type === 'l') {
    if (matches[1] === '?') {
      if (nums[0] === '25') {
        result.action = terminal => {
          if (type === 'l') terminal.state.cursorVisible = false
          else terminal.state.cursorVisible = true
        }
      }
    }
  }

  if (matches[1] !== "" || matches[3] !== "m") {
    return result
  }

  if (!markup) {
    return result
  }

  let self = this

  self.decoration = null

  while (nums.length > 0) {
    let num_str = nums.shift()
    let num = parseInt(num_str)

    if (isNaN(num) || num === 0) {
      self.fg = self.bg = self.decoration = 0
      self.fg = 7
    } else if (num === 1) {
      self.decoration = "bold"
    } else if (num === 2) {
      self.decoration = "dim"
    } else if (num == 3) {
      self.decoration = "italic"
    } else if (num == 4) {
      self.decoration = "underline"
    } else if (num == 5 || num === 6) {
      self.decoration = "blink"
    } else if (num === 7) {
      self.decoration = "reverse"
    } else if (num === 8) {
      self.decoration = "hidden"
    } else if (num === 9) {
      self.decoration = "strikethrough"
    } else if (num === 20) {
      self.decoration = 'fraktur'
    } else if (num == 39) {
      self.fg = 7
    } else if (num == 49) {
      self.bg = 0
    // Foreground color
    } else if ((num >= 30) && (num < 38)) {
      self.fg = num % 10
      // Foreground bright color
    } else if ((num >= 90) && (num < 98)) {
      self.fg = 8 + num % 10
      // Background color
    } else if ((num >= 40) && (num < 48)) {
      self.bg = num % 10
      // Background bright color
    } else if ((num >= 100) && (num < 108)) {
      self.bg = 8 + (num % 10)
    } else if (num === 38 || num === 48) { // extend color (38=fg, 48=bg)
      let is_foreground = (num === 38)
      if (nums.length >= 1) {
        let mode = nums.shift()
        if (mode === "5" && nums.length >= 1) { // palette color
          let palette_index = parseInt(nums.shift())
          if (palette_index >= 0 && palette_index <= 255) {
            if (is_foreground) {
              self.fg = this.PALETTE_COLORS[palette_index]
            } else {
              self.bg = this.PALETTE_COLORS[palette_index]
            }
          }
        } else if (mode === "2" && nums.length >= 3) { // true color
          let r = parseInt(nums.shift())
          let g = parseInt(nums.shift())
          let b = parseInt(nums.shift())
          if ((r >= 0 && r <= 255) && (g >= 0 && g <= 255) && (b >= 0 && b <= 255)) {
            let color = r + ", " + g + ", " + b
            if (is_foreground) {
              self.fg = color
            } else {
              self.bg = color
            }
          }
        }
      }
    }
  }

  if ((self.fg === null) && (self.bg === null) && (self.decoration === null)) {
    return result
  } else {
    let styles = []
    let classes = []
    let data = {}

    result.fg = self.fg
    result.bg = self.bg
    result.fg_truecolor = self.fg_truecolor
    result.bg_truecolor = self.bg_truecolor
    result.decoration = self.decoration
    result.was_processed = true

    return result
  }
}

processChunk (text, options, markup) {
  let self = this
  options = options || {}
  return this.processChunkJson(text, options, markup)
}
}

module.exports = Anser
