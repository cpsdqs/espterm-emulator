#! /bin/bash
echo -en '\x1b[2J'
if [ "${TERM_PROGRAM:='notespterm'}" != 'ESPTerm' ]; then
  echo -e '\x1b[41;97;1mrun this inside the emulator\x1b[0m'
else
  echo ''
fi

# lots of echoing:
echo -en '┌ESPTerm─Demo──'
echo -en '\x1b[31m31\x1b[32m32\x1b[33m33\x1b[34m34\x1b[35m35\x1b[36m36\x1b[37m37'
echo -en '\x1b[90m90\x1b[91m91\x1b[92m92\x1b[93m93\x1b[94m94\x1b[95m95\x1b[96m96\x1b[97m97'
echo -e '\x1b[0m─────────────┐'
echo -n '│'
for i in $(seq 0 56); do echo -n ' '; done
echo -n '│'
echo '    │││││││││'
echo -n '│'
echo -en '\x1b[1mBold \x1b[2mFaint \x1b[3mItalic \x1b[4mUnderline\x1b[0m \x1b[5mBlink'
echo -en ' \x1b[7mInverse\x1b[0m \x1b[9mStrike\x1b[0m \x1b[20mFraktur '
echo -n '│'
echo -e '  ──\x1b[100m         \x1b[0m──'
echo -n '│'
for i in $(seq 0 56); do echo -n ' '; done
echo -n '│'
echo -e '  ──\x1b[100;30m ESP8266 \x1b[0m──'
echo -n '└'
for i in $(seq 0 56); do echo -n '─'; done
echo -n '┤'
echo -e '  ──\x1b[100m         \x1b[0m──'
for i in $(seq 0 57); do echo -n ' '; done
echo -n '│'
echo -e '  ──\x1b[100;30m (@)#### \x1b[0m──'
echo -en ' \x1b[44;96m This is a static demo of the ESPTerm Web Interface    \x1b[0m  '
echo -n '│'
echo -e '  ──\x1b[100m         \x1b[0m──'
echo -en ' \x1b[44;96m                                                       \x1b[0m  '
echo -n '│'
echo '    │││││││││'
echo -e ' \x1b[44;96m Try the links beneath this screen to browse the menu. \x1b[0m  ♦'
echo -e ' \x1b[44;96m                                                       \x1b[0m'
echo -e ' \x1b[44;96m <°)))>< ESPTerm fully supports UTF-8 お は よ ー  ><(((°> \x1b[0m'
echo -e ' \x1b[44;96m                                                       \x1b[0m'
echo ''
echo -e ' \x1b[92mOther interesting features:\x1b[0m                        ↓'
echo ''
echo -en '   \x1b[32m- Almost full VT100 emulation  \x1b[35m() ()'
echo -e '\x1b[0m        Funguje tu čeština!'
echo -en  '   \x1b[34m- Xterm-like mouse tracking   \x1b[37m==\x1b[100m°.°\x1b[40m=='
echo -e ' \x1b[35m<---,'
echo -e "   \x1b[33m- File upload utility          \x1b[0m'' ''      \x1b[35mmouse"
echo -e '   \x1b[31m- User-friendly config interface'
echo -en '   \x1b[95m- Advanced WiFi & network settings'
for i in $(seq 0 16); do echo -n ' '; done
echo -e '\x1b[93mTry ESPTerm today!'
echo -en '   \x1b[37m- Built-in help page'
for i in $(seq 0 25); do echo -n ' '; done
echo -e '\x1b[36m-->  \x1b[93mPre-built binaries are'
for i in $(seq 0 29); do echo -n ' '; done
echo -e '\x1b[36mlink on the About page  \x1b[93mavailable on GitHub!'
