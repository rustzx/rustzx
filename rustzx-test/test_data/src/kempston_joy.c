#include "librustzx.h"

#define KEYS_COUNT 8

static unsigned char get_kempston_joy_state(void) __z88dk_callee {
    __asm__("in a, ($1f)");
    __asm__("mov l, a");
    __asm__("ret");
}

void main() {
    volatile unsigned char state = get_kempston_joy_state();
    rustzx_port_write_byte_hex(state);

    for (unsigned char i = 0; i < KEYS_COUNT * 2; ++i) {
        // Wait while rust changes kempston joy state via
        // public API
        rustzx_sync_with_host();
        state = get_kempston_joy_state();
        if (i != 0) {
            rustzx_port_write_char('>');
        }
        rustzx_port_write_byte_hex(state);
    }

    rustzx_sync_with_host();
}
