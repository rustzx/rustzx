#include "librustzx.h"

#define KEYS_COUNT 8


static unsigned char keyboard_state[8] = { 0, 0, 0, 0, 0, 0, 0, 0 };

static unsigned char query_keyboard_state(void) __z88dk_callee {
    __asm__("ld c, $FE");

    __asm__("ld b, $FE");
    __asm__("in a, (c)");
    __asm__("ld (_keyboard_state + 0), a");
    __asm__("ld b, $FD");
    __asm__("in a, (c)");
    __asm__("ld (_keyboard_state + 1), a");
    __asm__("ld b, $FB");
    __asm__("in a, (c)");
    __asm__("ld (_keyboard_state + 2), a");
    __asm__("ld b, $F7");
    __asm__("in a, (c)");
    __asm__("ld (_keyboard_state + 3), a");
    __asm__("ld b, $EF");
    __asm__("in a, (c)");
    __asm__("ld (_keyboard_state + 4), a");
    __asm__("ld b, $DF");
    __asm__("in a, (c)");
    __asm__("ld (_keyboard_state + 5), a");
    __asm__("ld b, $BF");
    __asm__("in a, (c)");
    __asm__("ld (_keyboard_state + 6), a");
    __asm__("ld b, $7F");
    __asm__("in a, (c)");
    __asm__("ld (_keyboard_state + 7), a");
    __asm__("ret");
}

void send_keyboard_state() {
    query_keyboard_state();

    for (int i = 0; i < 8; ++i) {
        // bitwise OR operation with each read byte is
        // required to ignore other ULA port content
        // (3 most significant bits). This is reuiqred
        // to make keyboard test independent from
        // other ULA functionality tests
        rustzx_port_write_byte_hex(keyboard_state[i] | 0xE0);
    }
}

void main() {
    // Respond to each sync request from the rust test
    for (;;) {
        rustzx_sync_with_host();
        send_keyboard_state();
        rustzx_port_write_char('\n');
    }
}
