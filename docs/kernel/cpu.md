# CPU Identification (`rust/src/cpu.rs`)

Modul kecil untuk identifikasi prosesor via instruksi `CPUID`, dipakai `lalaufetch` dan tersedia untuk komponen kernel lain.

## API

| Fungsi | Kembalian | Deskripsi |
| --- | --- | --- |
| `available()` | `bool` | Deteksi dukungan CPUID dengan men-toggle EFLAGS ID bit (bit 21) via `pushfd`/`popfd`; flag dikembalikan ke kondisi semula. |
| `vendor_string()` | `Option<String>` | 12 byte vendor dari leaf `0x0` (register `EBX:EDX:ECX`), mis. `GenuineIntel`. |
| `brand_string()` | `Option<String>` | 48 byte processor brand dari leaf `0x80000002..0x80000004`, mis. `QEMU Virtual CPU version 2.5+`. `None` bila CPUID tidak ada atau max extended leaf di bawah `0x80000004`. |

## Catatan

- Inline asm memakai sintaks Intel sesuai target `i686-unknown-linux-gnu` (relokasi statis, sehingga `ebx` aman dipakai sebagai operand).
- Tidak ada state global; tiap panggilan menjalankan `CPUID` ulang (operasi murah, hasil dependen pada CPU fisik).
