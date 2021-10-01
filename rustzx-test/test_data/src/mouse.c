#include <graphics.h>
#include <conio.h>

// We can't use here z88dk implementation is does not allow to
// do not well things like additional mouse buttons
static unsigned char io_kempston_state(void) __z88dk_callee {
    __asm__("ld a, $FA");
    __asm__("in a, ($DF)");
    __asm__("mov l, a");
    __asm__("ret");
}

static unsigned char io_kempston_mouse_x(void) __z88dk_callee {
    __asm__("ld a, $FB");
    __asm__("in a, ($DF)");
    __asm__("mov l, a");
    __asm__("ret");
}

static unsigned char io_kempston_mouse_y(void) __z88dk_callee {
    __asm__("ld a, $FF");
    __asm__("in a, ($DF)");
    __asm__("mov l, a");
    __asm__("ret");
}

static struct {
    unsigned char prev_x;
    unsigned char prev_y;
    unsigned char prev_state;
    unsigned char x;
    unsigned char y;
    unsigned char state;
} g_kemp_mouse;

static void kemp_poll(void) {
    g_kemp_mouse.prev_x = g_kemp_mouse.x;
    g_kemp_mouse.prev_y = g_kemp_mouse.y;
    g_kemp_mouse.prev_state = g_kemp_mouse.state;

    g_kemp_mouse.x = io_kempston_mouse_x();
    g_kemp_mouse.y = 255 - io_kempston_mouse_y();
    g_kemp_mouse.state = io_kempston_state();
}

static void kemp_init(void) {
    g_kemp_mouse.prev_x = g_kemp_mouse.x = io_kempston_mouse_x();
    g_kemp_mouse.prev_y = g_kemp_mouse.y = 255 - io_kempston_mouse_y();
    g_kemp_mouse.prev_state = g_kemp_mouse.state = io_kempston_state();
}

static unsigned char kemp_left_button_pressed(void) { return g_kemp_mouse.state & 0x01; }

static unsigned char kemp_right_button_pressed(void) { return g_kemp_mouse.state & 0x02; }

static unsigned char kemp_middle_button_pressed(void) { return g_kemp_mouse.state & 0x04; }

static unsigned char kemp_ext_button_pressed(void) { return g_kemp_mouse.state & 0x08; }

static char kemp_wheel_diff(void) {
    unsigned char prev_wheel = (g_kemp_mouse.prev_state & 0xF0);
    unsigned char wheel = (g_kemp_mouse.state & 0xF0);
    return (char)(wheel - prev_wheel) / 16;
}

static char kemp_x_diff(void) { return (char)(g_kemp_mouse.x - g_kemp_mouse.prev_x); }

static char kemp_y_diff(void) { return (char)(g_kemp_mouse.y - g_kemp_mouse.prev_y); }

static const char CURSOR_SPRITE[8] = { 0xE0, 0xF8, 0xFE, 0x7F, 0x7C, 0x3E, 0x37, 0x13 };

static int is_cursor_pixel_set(int cursor_x, int cursor_y, int screen_x, int screen_y) {
    if ((screen_x < cursor_x)
        || (screen_x >= cursor_x + 8)
        || (screen_y < cursor_y)
        || (screen_y >= cursor_y + 8))
    {
        return 0;
    }

    int rel_x = screen_x - cursor_x;
    int rel_y = screen_y - cursor_y;

    return CURSOR_SPRITE[rel_y] & (0x80 >> rel_x);
}

static void paint_cursor(int xpos, int ypos, int prev_xpos, int prev_ypos) {
    textcolor(BLACK);
    for (int y = 0; y < 8; y++) {
        for (int x = 0; x < 8; x++) {
            int screen_x = xpos + x;
            int screen_y = ypos + y;

            int prev_screen_x = prev_xpos + x;
            int prev_screen_y = prev_ypos + y;

            int is_prev_set = is_cursor_pixel_set(xpos, ypos, prev_screen_x, prev_screen_y);
            int is_now_set = is_cursor_pixel_set(xpos, ypos, screen_x, screen_y);

            // clear pixels in the previous cursor area
            if (!is_prev_set) {
                unplot(prev_screen_x, prev_screen_y);
            }

            // set pixels in the new pixels area
            if (is_now_set) {
                plot(screen_x, screen_y);
            }
        }
    }
}

static void draw_button_box(unsigned char index, unsigned char pressed) {
    if (pressed) {
        textcolor(RED); drawb(32 * index, 0, 3, 3);
    } else {
        textcolor(WHITE); drawb(32 * index, 0, 3, 3);
    }
}

static void draw_scroll_box(unsigned char prev_ypos, unsigned char ypos) {
    textcolor(WHITE);
    drawb(0, prev_ypos, 3, 3);
    textcolor(RED);
    drawb(0, ypos, 3, 3);
}

void main() {
    cclg();
    kemp_init();
    unsigned char xpos = 0;
    unsigned char ypos = 0;
    unsigned char prev_xpos = 0;
    unsigned char prev_ypos = 0;

    unsigned char wheel = 0;

    while(1) {
        kemp_poll();

        short virtual_x = (short)xpos + kemp_x_diff();
        short virtual_y = (short)ypos + kemp_y_diff();
        if (virtual_y > 191) virtual_y = 1;
        if (virtual_y < 0) virtual_y = 192;

        prev_xpos = xpos;
        prev_ypos = ypos;
        xpos = (unsigned char) virtual_x;
        ypos = (unsigned char) virtual_y;

        paint_cursor(xpos, ypos, prev_xpos, prev_ypos);

        char wheel_diff = kemp_wheel_diff();
        if (wheel_diff != 0) {
            unsigned char prev_wheel = wheel;
            if (wheel_diff > 0) {
                if ((short)wheel + wheel_diff < 192)
                    wheel = (short)wheel + wheel_diff;
                else
                    wheel = 191;
            }
            if (wheel_diff < 0) {
                if ((short)wheel + wheel_diff >= 0)
                    wheel = (short)wheel + wheel_diff;
                else
                    wheel = 0;
            }
            draw_scroll_box(prev_wheel, wheel);
        }

        draw_button_box(1, kemp_left_button_pressed());
        draw_button_box(2, kemp_right_button_pressed());
        draw_button_box(3, kemp_middle_button_pressed());
        draw_button_box(4, kemp_ext_button_pressed());
    }
}
