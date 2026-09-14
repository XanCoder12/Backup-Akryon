#ifndef AKRYON_HAL_MOUSE_H
#define AKRYON_HAL_MOUSE_H

#include "types.h"

#define MOUSE_BTN_LEFT   (1 << 0)
#define MOUSE_BTN_RIGHT  (1 << 1)
#define MOUSE_BTN_MIDDLE (1 << 2)

void mouse_init(void);
void mouse_set_bounds(int32_t max_x, int32_t max_y);
void mouse_get_state(int32_t* x, int32_t* y, uint8_t* buttons);
int32_t mouse_get_x(void);
int32_t mouse_get_y(void);
uint8_t mouse_get_buttons(void);
uint32_t mouse_get_event_count(void);

#endif // AKRYON_HAL_MOUSE_H
