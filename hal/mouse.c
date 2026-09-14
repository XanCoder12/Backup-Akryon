#include "mouse.h"
#include "io.h"
#include "isr.h"
#include "serial.h"

static volatile int32_t mouse_x = 400;
static volatile int32_t mouse_y = 300;
static volatile int32_t mouse_max_x = 1023;
static volatile int32_t mouse_max_y = 767;
static volatile uint8_t mouse_buttons = 0;
static volatile uint32_t mouse_event_count = 0;

static uint8_t packet[3];
static uint8_t packet_idx = 0;

static inline void mouse_wait_write(void) {
    uint32_t timeout = 100000;
    while ((inb(0x64) & 0x02) && --timeout) {
        io_wait();
    }
}

static inline void mouse_wait_read(void) {
    uint32_t timeout = 100000;
    while (!(inb(0x64) & 0x01) && --timeout) {
        io_wait();
    }
}

static void mouse_write(uint8_t val) {
    mouse_wait_write();
    outb(0x64, 0xD4);
    mouse_wait_write();
    outb(0x60, val);
}

static uint8_t mouse_read(void) {
    mouse_wait_read();
    return inb(0x60);
}

static void mouse_callback(registers_t* regs) {
    (void)regs;
    uint8_t status = inb(0x64);
    if (!(status & 0x01)) {
        return;
    }

    uint8_t b = inb(0x60);

    if (packet_idx == 0) {
        if (!(b & 0x08)) {
            return;
        }
    }

    packet[packet_idx++] = b;

    if (packet_idx == 3) {
        packet_idx = 0;

        if (packet[0] & 0xC0) {
            return;
        }

        int32_t dx = (int32_t)packet[1];
        if (packet[0] & 0x10) {
            dx |= (int32_t)0xFFFFFF00;
        }

        int32_t dy = (int32_t)packet[2];
        if (packet[0] & 0x20) {
            dy |= (int32_t)0xFFFFFF00;
        }

        mouse_buttons = packet[0] & 0x07;

        int32_t nx = mouse_x + dx;
        int32_t ny = mouse_y - dy;

        if (nx < 0) nx = 0;
        if (nx > mouse_max_x) nx = mouse_max_x;
        if (ny < 0) ny = 0;
        if (ny > mouse_max_y) ny = mouse_max_y;

        mouse_x = nx;
        mouse_y = ny;
        mouse_event_count++;
    }
}

void mouse_init(void) {
    mouse_wait_write();
    outb(0x64, 0xA8);

    mouse_wait_write();
    outb(0x64, 0x20);
    uint8_t status = mouse_read();

    status |= 0x02;
    status &= ~0x20;

    mouse_wait_write();
    outb(0x64, 0x60);
    mouse_wait_write();
    outb(0x60, status);

    mouse_write(0xF6);
    (void)mouse_read();

    mouse_write(0xF4);
    (void)mouse_read();

    isr_register_handler(44, mouse_callback);
    serial_puts("[Akryon HAL] PS/2 Mouse streaming enabled on IRQ 12 (vector 44).\n");
}

void mouse_set_bounds(int32_t max_x, int32_t max_y) {
    mouse_max_x = max_x > 0 ? max_x : 0;
    mouse_max_y = max_y > 0 ? max_y : 0;
    if (mouse_x > mouse_max_x) mouse_x = mouse_max_x;
    if (mouse_y > mouse_max_y) mouse_y = mouse_max_y;
}

void mouse_get_state(int32_t* x, int32_t* y, uint8_t* buttons) {
    if (x) *x = mouse_x;
    if (y) *y = mouse_y;
    if (buttons) *buttons = mouse_buttons;
}

int32_t mouse_get_x(void) {
    return mouse_x;
}

int32_t mouse_get_y(void) {
    return mouse_y;
}

uint8_t mouse_get_buttons(void) {
    return mouse_buttons;
}

uint32_t mouse_get_event_count(void) {
    return mouse_event_count;
}
