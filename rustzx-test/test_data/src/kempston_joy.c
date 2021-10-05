#include "librustzx.h"

static unsigned char get_kempston_joy_state(void) __z88dk_callee {
    __asm__("in a, ($1f)");
    __asm__("mov l, a");
    __asm__("ret");
}

void main() {
    // Respond to each sync request from the rust test
    for (;;) {
        rustzx_sync_with_host();
        rustzx_port_write_byte_hex(get_kempston_joy_state());
        rustzx_port_write_char(',');
    }
}
