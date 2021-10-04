#include "librustzx.h"

static const char HEX_ALPHABET[16] = {
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'
};

static unsigned char read_debug_port(void) __z88dk_callee {
    __asm__("ld a, $CC");
    __asm__("in a, ($CC)");
    __asm__("mov l, a");
    __asm__("ret");
}

static volatile unsigned char _rustzx_debug_port_tmp = 0;

static unsigned char write_debug_port(void) __z88dk_callee {
    __asm__("ld a, (__rustzx_debug_port_tmp)");
    __asm__("ld b, $CC");
    __asm__("ld c, $CC");
    __asm__("out (c), a");
    __asm__("ret");
}

void rustzx_port_write_str(char* s) {
    while (*s) {
        _rustzx_debug_port_tmp = *(s++);
        write_debug_port();
    }
}

void rustzx_port_write_char(char x) {
    _rustzx_debug_port_tmp = x;
    write_debug_port();
}

void rustzx_port_write_byte_hex(unsigned char x) {
    _rustzx_debug_port_tmp = HEX_ALPHABET[(x & 0xF0) >> 4];
    write_debug_port();
    _rustzx_debug_port_tmp = HEX_ALPHABET[x & 0x0F];
    write_debug_port();
}

unsigned char rustzx_port_read_byte() {
    return read_debug_port();
}

void rustzx_sync_with_host() {
    while (!read_debug_port()) {}
    _rustzx_debug_port_tmp = 1;
    write_debug_port();
}
