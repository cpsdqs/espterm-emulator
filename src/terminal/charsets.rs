// Adapted from character_sets.h in espterm-firmware

pub struct CodePageData {
    pub begin: u32,
    pub end: u32,
    pub data: &'static [char],
}

///
/// translates VT100 ACS escape codes to Unicode values.
/// Based on rxvt-unicode screen.C table.
///
pub const CODE_PAGE_0: CodePageData = CodePageData {
    begin: 96,
    end: 126,
    data: &[
        '♦', // 0x2666   96  `
        '▒', // 0x2592   97  a
        '␉', // 0x2409   98  b
        '␌', // 0x240c   99  c   FF
        '␍', // 0x240d   100 d   CR
        '␊', // 0x240a   101 e   LF
        '°',  // 0x00b0   102 f
        '±',  // 0x00b1   103 g
        '␤', // 0x2424   104 h   NL
        '␋', // 0x240b   105 i   VT
        '┘', // 0x2518   106 j
        '┐', // 0x2510   107 k
        '┌', // 0x250c   108 l
        '└', // 0x2514   109 m
        '┼', // 0x253c   110 n
        '⎺', // 0x23ba   111 o
        '⎻', // 0x23bb   112 p
        '─', // 0x2500   113 q
        '⎼', // 0x23bc   114 r
        '⎽', // 0x23bd   115 s
        '├', // 0x251c   116 t
        '┤', // 0x2524   117 u
        '┴', // 0x2534   118 v
        '┬', // 0x252c   119 w
        '│', // 0x2502   120 x
        '≤', // 0x2264   121 y
        '≥', // 0x2265   122 z
        'π',  // 0x03c0   123 {
        '≠', // 0x2260   124 |
        '£',  // 0x20a4   125 }
        '·',  // 0x00b7   126 ~
    ],
};

// DOS, thin and double lines, arrows, angles, diagonals, thick border
pub const CODE_PAGE_1: CodePageData = CodePageData {
    begin: 33,
    end: 126,
    data: &[
        '☺',      // 0x263A,  33  !   (1)  - low ASCII symbols from DOS, moved to +32
        '☻',      // 0x263B,  34  "   (2)
        '♥',      // 0x2665,  35  #   (3)
        '♦',      // 0x2666,  36  $   (4)
        '♣',      // 0x2663,  37  %   (5)
        '♠',      // 0x2660,  38  &   (6)
        '•',      // 0x2022,  39  '   (7)  - inverse dot and circle left out, can be done with SGR
        '⌛',      // 0x231B,  40  (   - hourglass (timer icon)
        '○',      // 0x25CB,  41  )   (9)
        '↯',      // 0x21AF,  42  *   - electricity (lightning monitor...)
        '♪',      // 0x266A,  43  +   (13)
        '♫',      // 0x266B,  44  ,   (14)
        '☼',      // 0x263C,  45  -   (15)
        '⌂',      // 0x2302,  46  .   (127)
        '☢',      // 0x2622,  47  /   - radioactivity (geiger counter...)
        '░', // 0x2591,  48  0   (176) - this block is kept aligned and ordered from DOS, moved -128
        '▒', // 0x2592,  49  1   (177)
        '▓', // 0x2593,  50  2   (178)
        '│', // 0x2502,  51  3   (179)
        '┤', // 0x2524,  52  4   (180)
        '╡', // 0x2561,  53  5   (181)
        '╢', // 0x2562,  54  6   (182)
        '╖', // 0x2556,  55  7   (183)
        '╕', // 0x2555,  56  8   (184)
        '╣', // 0x2563,  57  9   (185)
        '║', // 0x2551,  58  :   (186)
        '╗', // 0x2557,  59  ;   (187)
        '╝', // 0x255D,  60  <   (188)
        '╜', // 0x255C,  61  =   (189)
        '╛', // 0x255B,  62  >   (190)
        '┐', // 0x2510,  63  ?   (191)
        '└', // 0x2514,  64  @   (192)
        '┴', // 0x2534,  65  A   (193)
        '┬', // 0x252C,  66  B   (194)
        '├', // 0x251C,  67  C   (195)
        '─', // 0x2500,  68  D   (196)
        '┼', // 0x253C,  69  E   (197)
        '╞', // 0x255E,  70  F   (198)
        '╟', // 0x255F,  71  G   (199)
        '╚', // 0x255A,  72  H   (200)
        '╔', // 0x2554,  73  I   (201)
        '╩', // 0x2569,  74  J   (202)
        '╦', // 0x2566,  75  K   (203)
        '╠', // 0x2560,  76  L   (204)
        '═', // 0x2550,  77  M   (205)
        '╬', // 0x256C,  78  N   (206)
        '╧', // 0x2567,  79  O   (207)
        '╨', // 0x2568,  80  P   (208)
        '╤', // 0x2564,  81  Q   (209)
        '╥', // 0x2565,  82  R   (210)
        '╙', // 0x2559,  83  S   (211)
        '╘', // 0x2558,  84  T   (212)
        '╒', // 0x2552,  85  U   (213)
        '╓', // 0x2553,  86  V   (214)
        '╫', // 0x256B,  87  W   (215)
        '╪', // 0x256A,  88  X   (216)
        '┘', // 0x2518,  89  Y   (217)
        '┌', // 0x250C,  90  Z   (218)
        '█', // 0x2588,  91  [   (219)
        '▄', // 0x2584,  92  \   (220)
        '▌', // 0x258C,  93  ]   (221)
        '▐', // 0x2590,  94  ^   (222)
        '▀', // 0x2580,  95  _   (223)
        '↕', // 0x2195,  96  `   (18)  - moved from low DOS ASCII
        '↑', // 0x2191,  97  a   (24)
        '↓', // 0x2193,  98  b   (25)
        '→', // 0x2192,  99  c   (26)
        '←', // 0x2190,  100 d   (27)
        '↔', // 0x2194,  101 e   (29)
        '▲', // 0x25B2,  102 f   (30)
        '▼', // 0x25BC,  103 g   (31)
        '►', // 0x25BA,  104 h   (16)
        '◄', // 0x25C4,  105 i   (17)
        '◢', // 0x25E2,  106 j   - added for slanted corners
        '◣', // 0x25E3,  107 k
        '◤', // 0x25E4,  108 l
        '◥', // 0x25E5,  109 m
        '╭', // 0x256D,  110 n   - rounded corners
        '╮', // 0x256E,  111 o
        '╯', // 0x256F,  112 p
        '╰', // 0x2570,  113 q
        '╱', // 0x0, 114 r - right up diagonal
        '╲', // 0x0, 115 s - right down diagonal
        '╳', // 0x0, 116 t
        '↺', // 0x0, 117 u
        '↻', // 0x0, 118 v
        '¶',  // 0x0, 119 w
        '⏻', // 0x0, 120 x
        '\u{e0b0}', // powerline right triangle (filled), 121 y
        '\u{e0b1}', // powerline right triangle (hollow), 122   z
        '\u{e0b2}', // powerline left triangle (filled), 123    {
        '\u{e0b3}', // powerline left triangle (hollow), 124    |   - reserved
        '✔', // 0x2714,  125 }   - checkboxes or checklist items
        '✘', // 0x2718,  126 ~
    ],
};

// blocks, thick and split lines, line butts
#[allow(dead_code)]
pub const CODE_PAGE_2: CodePageData = CodePageData {
    begin: 33,
    end: 126,
    data: &[
        '▁', // 0x2581,  33  ! - those are ordered this way to allow easy calculating of the right code (for graphs)
        '▂', // 0x2582,  34  "
        '▃', // 0x2583,  35  #
        '▄', // 0x2584,  36  $
        '▅', // 0x2585,  37  %
        '▆', // 0x2586,  38  &
        '▇', // 0x2587,  39  ' - 7-eighths
        '█', // 0x2588,  40  ( - full block, shared by both sequences
        '▉', // 0x2589,  41  ) - those grow thinner, to re-use the full block
        '▊', // 0x258A,  42  *
        '▋', // 0x258B,  43  +
        '▌', // 0x258C,  44  ,
        '▍', // 0x258D,  45  -
        '▎', // 0x258E,  46  .
        '▏', // 0x258F,  47  /
        '▔', // 0x2594,  48  0 - complementary symbols
        '▕', // 0x2595,  49  1
        '▐', // 0x2590,  50  2
        '▀', // 0x2580,  51  3
        '▘', // 0x2598,  52  4 - top-left, top-right, bottom-right, bottom-left
        '▝', // 0x259D,  53  5
        '▗', // 0x2597,  54  6
        '▖', // 0x2596,  55  7
        '▟', // 0x259F,  56  8
        '▙', // 0x2599,  57  9
        '▛', // 0x259B,  58  :
        '▜', // 0x259C,  59  ;
        '▞', // 0x259E,  60  < - complementary diagonals
        '▚', // 0x259A,  61  =
        '━', // 0x,  62  > - here are thick and thin lines and their joins. it's really quite arbitrary, based on the unicode order, excluding single lines
        '┃', // 0x,  63  ?
        '┍', // 0x,  64  @
        '┎', // 0x,  65  A
        '┏', // 0x,  66  B
        '┑', // 0x,  67  C
        '┒', // 0x,  68  D
        '┓', // 0x,  69  E
        '┕', // 0x,  70  F
        '┖', // 0x,  71  G
        '┗', // 0x,  72  H
        '┙', // 0x,  73  I
        '┚', // 0x,  74  J
        '┛', // 0x,  75  K
        '┝', // 0x,  76  L
        '┞', // 0x,  77  M
        '┟', // 0x,  78  N
        '┠', // 0x,  79  O
        '┡', // 0x,  80  P
        '┢', // 0x,  81  Q
        '┣', // 0x,  82  R
        '┥', // 0x,  83  S
        '┦', // 0x,  84  T
        '┧', // 0x,  85  U
        '┨', // 0x,  86  V
        '┩', // 0x,  87  W
        '┪', // 0x,  88  X
        '┫', // 0x,  89  Y
        '┭', // 0x,  90  Z
        '┮', // 0x,  91  [
        '┯', // 0x,  92  \ .
        '┰', // 0x,  93  ]
        '┱', // 0x,  94  ^
        '┲', // 0x,  95  _
        '┳', // 0x,  96  `
        '┵', // 0x,  97  a
        '┶', // 0x,  98  b
        '┷', // 0x,  99  c
        '┸', // 0x,  100 d
        '┹', // 0x,  101 e
        '┺', // 0x,  102 f
        '┻', // 0x,  103 g
        '┽', // 0x,  104 h
        '┾', // 0x,  105 i
        '┿', // 0x,  106 j
        '╀', // 0x,  107 k
        '╁', // 0x,  108 l
        '╂', // 0x,  109 m
        '╃', // 0x,  110 n
        '╄', // 0x,  111 o
        '╅', // 0x,  112 p
        '╆', // 0x,  113 q
        '╇', // 0x,  114 r
        '╈', // 0x,  115 s
        '╉', // 0x,  116 t
        '╊', // 0x,  117 u
        '╋', // 0x,  118 v
        '╴', // 0x,  119 w - butts
        '╵', // 0x,  120 x
        '╶', // 0x,  121 y
        '╷', // 0x,  122 z
        '╸', // 0x,  123 {
        '╹', // 0x,  124 |
        '╺', // 0x,  125 }
        '╻', // 0x,  126 ~
    ],
};

// dashed lines, split straight lines
#[allow(dead_code)]
pub const CODE_PAGE_3: CodePageData = CodePageData {
    begin: 33,
    end: 48,
    data: &[
        '╌', // 0x,  33  !
        '┄', // 0x,  34  "
        '┈', // 0x,  35  #
        '╍', // 0x,  36  $
        '┅', // 0x,  37  %
        '┉', // 0x,  38  &
        '╎', // 0x,  39  '
        '┆', // 0x,  40  (
        '┊', // 0x,  41  )
        '╏', // 0x,  42  *
        '┇', // 0x,  43  +
        '┋', // 0x,  44  ,
        '╼', // 0x,  45  -
        '╽', // 0x,  46  .
        '╾', // 0x,  47  /
        '╿', // 0x,  48  0
    ],
};
