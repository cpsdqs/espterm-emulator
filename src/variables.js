let packageInfo = require('../package')

let pad = (s, l, a, b) => {
  s = s.toString()
  let h = s.length
  let o = Math.floor((l - h) / 2)
  let r = (t, y) => (y || '').repeat(Math.max(0, t))
  let c = a && b
  return r(c ? o : a ? l - h : 0, a) + s + r(c ? l - o - h : b ? l - h : 0, b)
}

let getISODate = function () {
  let date = new Date()
  return pad(date.getUTCFullYear(), 4, '0') + '-' +
    pad(date.getUTCMonth(), 2, '0') + '-' +
    pad(date.getUTCDate(), 2, '0')
}

let getShortISOTime = function () {
  let date = new Date()
  return pad(date.getUTCHours(), 2, '0') + ':' +
    pad(date.getUTCMinutes(), 2, '0')
}

// various variables used by the TPL files
module.exports = {
  theme: 0,
  labels_seq: 'T<init from JS>\x01\x01\x01\x01\x01',
  default_fg: 7,
  default_bg: 0,
  want_all_fn: 0,
  backdrop: '',

  vers_fw: packageInfo.version,
  date: '— well, not built, but started ' + getISODate(),
  time: getShortISOTime(),
  vers_httpd: `(this isn't httpd. This is Node ${process.version})`,
  vers_sdk: `(this doesn't use any IoT SDK)`,
  githubrepo: 'https://github.com/ESPTerm/espterm-firmware',
  githubrepo_front: 'https://github.com/ESPTerm/espterm-front-end',
  hash_backend: 'f7edbf19',
  hash_frontend: '75496b8b',

  sta_dhcp_enable: 1,
  sta_addr_ip: 'NaN.NaN.NaN.Batman',
  sta_addr_mask: 'NaN.NaN.NaN.Batman',
  sta_addr_gw: 'NaN.NaN.NaN.Batman',
  ap_addr_mask: 'NaN.NaN.NaN.Batman',
  ap_addr_ip: 'NaN.NaN.NaN.Batman',
  ap_dhcp_start: 'NaN.NaN.NaN.Batman',
  ap_dhcp_end: 'NaN.NaN.NaN.Batman',
  ap_dhcp_time: 420,
  sta_mac: '00:21:47:48:36:47',
  ap_mac: '01:21:47:48:36:47',

  uart_baud: 115200,
  uart_parity: 2,
  uart_stopbits: 1,

  term_width: 80,
  term_height: 25,
  term_title: 'ESPTerm',
  btn1: '1',
  btn2: '2',
  btn3: '3',
  btn4: '4',
  btn5: '5',
  parser_tout_ms: 10,
  display_tout_ms: 15,
  display_cooldown_ms: 35,
  bm1: '01,121',
  bm2: '01,110',
  bm3: '',
  bm4: '',
  bm5: '05',
  fn_alt_mode: 1,
  show_buttons: 1,
  show_config_links: 1,
  loopback: 0,
  cursor_shape: 1,
  crlf_mode: 0,
  allow_decopt_12: 0,
  ascii_debug: 0,
  debugbar: 0,

  ap_enable: 1,
  ap_ssid: 'THIS-wontactuallydoanything',
  ap_password: '',
  ap_channel: 7,
  tpw: 60,
  ap_hidden: 0,
  sta_enable: 1,
  sta_ssid: `Node ${process.version}`,
  sta_password: '',
  sta_active_ip: 'not an IP',
  sta_active_ssid: `Node ${process.version}`,
  access_name: process.env['USER'],
  pwlock: 0,
  overclock: 1,
  def_access_pw: 'rainbows',
  def_admin_pw: 'rainbows'
}
