#ifndef NYXARA_HAL_IO_H
#define NYXARA_HAL_IO_H

#include "types.h"

void outb(uint16_t port, uint8_t val);
uint8_t inb(uint16_t port);
void outw(uint16_t port, uint16_t val);
uint16_t inw(uint16_t port);
void outl(uint16_t port, uint32_t val);
uint32_t inl(uint16_t port);
void io_wait(void);
void cli(void);
void sti(void);
void hlt(void);

#endif // NYXARA_HAL_IO_H
