use std::collections::HashMap;

pub fn defaults() -> HashMap<String, String> {
    let mut vars = HashMap::new();

    macro_rules! insert {
        ($($key:ident : $val:expr),+,) => {
            $(
                vars.insert(stringify!($key).into(), $val.into());
            )+
        }
    }

    insert! {
        want_all_fn: "0",
        theme: "0",
        default_fg: "7",
        default_bg: "0",
        font_stack: "Inconsolata",
        font_size: "12",
        backdrop: "",
        width: "100",
        height: "36",
        title: "ESPTerm",
        button_count: "0",
        show_buttons: "0",
        btn1: "1",
        btn2: "2",
        btn3: "3",
        btn4: "4",
        btn5: "5",
        bc1: "0",
        bc2: "0",
        bc3: "0",
        bc4: "0",
        bc5: "0",
        parser_tout_ms: "10",
        display_tout_ms: "15",
        display_cooldown_ms: "35",
        bm1: "01,121",
        bm2: "01,110",
        bm3: "",
        bm4: "",
        bm5: "05",
        crlf_mode: "",
        loopback: "0",
        debugbar: "0",
        ascii_debug: "0",
        fn_alt_mode: "1",
        show_config_links: "1",
        allow_decopt_12: "0",
        cursor_shape: "1",
        uart_baudrate: "115200",
        uart_parity: "2",
        uart_stopbits: "1",
        ap_enable: "1",
        ap_ssid: "horse",
        ap_password: "",
        ap_channel: "7",
        tpw: "60",
        ap_hidden: "0",
        sta_enable: "1",
        sta_ssid: "horse",
        sta_password: "",
        sta_active_ip: "NaN.NaN.NaN.Horse",
        sta_active_ssid: "horse",
        sta_dhcp_enable: "1",
        sta_addr_ip: "NaN.NaN.NaN.Batman",
        sta_addr_mask: "NaN.NaN.NaN.Batman",
        sta_addr_gw: "NaN.NaN.NaN.Batman",
        ap_addr_mask: "NaN.NaN.NaN.Batman",
        ap_addr_ip: "NaN.NaN.NaN.Batman",
        ap_dhcp_start: "NaN.NaN.NaN.Batman",
        ap_dhcp_end: "NaN.NaN.NaN.Batman",
        ap_dhcp_time: "0",
        sta_mac: "00:21:47:48:36:47",
        ap_mac: "01:21:47:48:36:47",
        overclock: "1",
        def_access_pw: "horse",
        def_admin_pw: "horse",
        pwlock: "0",
        access_name: "root",

        vers_fw: env!("CARGO_PKG_VERSION"),
        date: "some day",
        time: "some time",
        githubrepo: "https://github.com/ESPTerm/espterm-firmware",
        githubrepo_front: "https://github.com/ESPTerm/espterm-front-end",
        hash_backend: "f7edbf19",
        hash_frontend: "75496b8b",
        vers_httpd: "ery N/A",
        vers_sdk: "ery N/A",
    };

    vars
}
