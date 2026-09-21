# NyxaraOS Commands

Dokumentasi command bawaan shell NyxaraOS.

> Catatan: VFS saat ini masih berbasis RAM. File dan direktori belum bertahan setelah reboot.

## Bantuan dan informasi sistem

### `help`

```text
help
```

Menampilkan daftar command bawaan shell.

### `about`

```text
about
```

Menampilkan informasi umum tentang NyxaraOS, arsitektur kernel, dan komponen sistem.

### `sysinfo`

```text
sysinfo
```

Menampilkan informasi status CPU, protected mode, PIT, interrupt, dan serial port.

### `free` / `meminfo`

```text
free
meminfo
```

Menampilkan penggunaan physical memory, status heap, paging, CR3, mapped pages, dan demand paging.

### `uptime`

```text
uptime
```

Menampilkan waktu sistem sejak boot.

### `date`

```text
date
```

Menampilkan tanggal dari RTC/CMOS.

### `time`

```text
time
```

Menampilkan waktu dari RTC/CMOS.

### `clear`

```text
clear
```

Membersihkan layar terminal.

## Direktori dan filesystem

### `pwd`

```text
pwd
```

Menampilkan current working directory.

Contoh:

```text
nyxara /projects > pwd
/projects
```

### `cd`

```text
cd <directory>
```

Berpindah ke direktori lain. Mendukung path absolut, path relatif, `.`, dan `..`.

Contoh:

```text
cd /
cd projects
cd ./src
cd ..
cd /etc
```

Tanpa argumen, `cd` kembali ke root `/`.

### `mkdir`

```text
mkdir <directory>
```

Membuat direktori baru. Direktori parent harus sudah ada.

Contoh:

```text
mkdir projects
mkdir projects/src
```

### `rm`

```text
rm <file>
```

Menghapus file. Command ini tidak menghapus direktori dan tidak mendukung recursive delete.

### `rmdir`

```text
rmdir <empty-directory>
```

Menghapus direktori yang kosong. Root, current directory, dan direktori yang masih berisi file tidak dapat dihapus.

### `cp`

```text
cp <source> <destination>
```

Menyalin file ke path baru. Command menolak tujuan yang sudah ada agar tidak menimpa data secara tidak sengaja. Jika tujuan adalah direktori, nama file sumber digunakan di dalam direktori tersebut.

### `mv`

```text
mv <source> <destination>
```

Memindahkan atau mengganti nama file. Command menolak tujuan yang sudah ada dan hanya bekerja pada file.

### `stat`

```text
stat <path>
```

Menampilkan path canonical, tipe inode, dan ukuran path yang diberikan.

### `ls`

```text
ls [path]
```

Menampilkan isi direktori. Jika path tidak diberikan, command menampilkan isi current directory.

Contoh:

```text
ls
ls /
ls /etc
ls ..
```

Direktori ditandai dengan `<DIR>`.

### `cat`

```text
cat <file>
```

Menampilkan isi file teks. Jika file berisi data non-teks, isinya ditampilkan dalam format hexadecimal.

Contoh:

```text
cat readme.txt
cat /etc/network.conf
```

### `touch`

```text
touch <file>
```

Membuat file kosong. Jika file sudah ada, isinya dikosongkan.

Contoh:

```text
touch notes.txt
touch projects/notes.txt
```

### `write`

```text
write <file> <text>
```

Menulis teks ke file dan menambahkan newline di akhir isi.

Contoh:

```text
write notes.txt Hello Nyxara
cat notes.txt
```

### `mway`

```text
mway <file>
```

Membuka file menggunakan text editor penuh layar bawaan NyxaraOS.

Shortcut utama di editor:

| Shortcut | Fungsi |
|---|---|
| `Ctrl+S` | Simpan file |
| `Ctrl+Q` | Keluar dari editor |
| `Shift+Arrow` | Memilih teks |
| `Shift+Home` | Memilih hingga awal baris |
| `Shift+End` | Memilih hingga akhir baris |
| `Backspace` / `Delete` | Menghapus selection |
| `Ctrl+Arrow` | Berpindah per kata |
| `Alt+Backspace` | Menghapus kata sebelumnya |
| `Alt+Delete` | Menghapus kata berikutnya |

## Shell dan utilitas dasar

### `echo`

```text
echo <text>
```

Menampilkan teks ke terminal.

Contoh:

```text
echo Hello Nyxara
```

### `color`

```text
color <foreground> <background>
```

Mengubah warna foreground dan background terminal. Nilai warna berada pada rentang `0..15`.

Contoh:

```text
color 15 0
color 14 1
```

### `calc`

```text
calc <a op b>
```

Menghitung operasi aritmetika integer.

Contoh:

```text
calc 10 + 5
calc 8 * 4
```

### `history`

History command tersimpan secara internal di `.nyxara_history` melalui VFS. Navigasi history dilakukan dengan tombol `Up` dan `Down`.

`Ctrl+R` digunakan untuk mencari command sebelumnya berdasarkan teks yang sedang ada di line editor.

### `syscall`

```text
syscall
```

Menjalankan pengujian system call Unix `int 0x80`, termasuk syscall write dan getpid.

### `reboot`

```text
reboot
```

Merestart sistem melalui controller keyboard 8042.

### `panic`

```text
panic [message]
```

Memicu kernel panic secara sengaja untuk menguji panic handler.

Contoh:

```text
panic test page fault
```

Gunakan hanya untuk debugging.

### `lalaufetch`

```text
lalaufetch
```

Menampilkan informasi sistem dengan tampilan bergaya `neofetch`.

## Networking

### `ifconfig` / `netinfo`

```text
ifconfig
ifconfig eth0
ifconfig set <ip> <netmask> <gateway>
netinfo
```

Menampilkan status interface RTL8139 `eth0`, alamat IP, netmask, gateway, DNS, MAC address, dan statistik paket.

Contoh:

```text
ifconfig eth0
ifconfig set 192.168.1.20 255.255.255.0 192.168.1.1
```

### `dhcp`

```text
dhcp
```

Meminta konfigurasi jaringan secara otomatis melalui DHCP.

### `dns` / `nslookup`

```text
dns <hostname>
nslookup <hostname>
```

Melakukan resolusi hostname menjadi alamat IPv4 melalui DNS.

Contoh:

```text
dns example.com
```

### `ping`

```text
ping <host>
```

Mengirim ICMP echo request ke hostname atau alamat IPv4.

Contoh:

```text
ping 8.8.8.8
ping example.com
```

### `arp`

```text
arp
arp -a
arp -c
arp clear
```

Menampilkan atau membersihkan ARP cache.

- `arp` dan `arp -a`: menampilkan tabel ARP.
- `arp -c` dan `arp clear`: menghapus semua entry ARP.

### `netstat`

```text
netstat
```

Menampilkan status interface, statistik traffic, jumlah socket TCP, dan jumlah entry ARP.

### `curl` / `fetch`

```text
curl <url>
fetch <url>
```

Mengambil konten melalui HTTP menggunakan network stack NyxaraOS.

Contoh:

```text
curl http://example.com/
```

### `httpd`

```text
httpd [port]
```

Menjalankan HTTP server sederhana dari dalam kernel. Port default adalah `8080`.

Contoh:

```text
httpd
httpd 8000
```

Tekan `Ctrl+C` untuk menghentikan server.

### `nc`

```text
nc [-u] <ip> <port>
```

Menjalankan client Netcat untuk mengirim payload melalui TCP atau UDP.

Contoh:

```text
nc 192.168.1.10 9000
nc -u 192.168.1.10 9000
```

## Memory dan kernel debugging

### `vmm`

```text
vmm <subcommand>
```

Mengelola dan menguji Virtual Memory Manager.

Subcommand yang tersedia:

```text
vmm info
vmm status
vmm test
vmm map <virtual_address> <physical_address>
vmm fault
```

- `vmm info` / `vmm status`: menampilkan status paging, CR3, mapped pages, dan demand paging.
- `vmm test`: menjalankan pengujian paging dan demand paging.
- `vmm map`: memetakan virtual address ke physical address.
- `vmm fault`: memicu page fault secara sengaja.

Gunakan `vmm map` dan `vmm fault` hanya untuk debugging kernel.

## Perangkat interaktif

### `mouse`

```text
mouse
mouse test
```

Menampilkan status hardware mouse PS/2. Mode `test` menampilkan posisi pointer dan status tombol secara interaktif.

Tekan `q` atau `Esc` untuk keluar dari mode test.

### `paint`

```text
paint
```

Membuka canvas gambar sederhana menggunakan mouse.

- Gerakkan mouse untuk menggambar.
- Gunakan tombol dan shortcut yang ditampilkan di canvas untuk memilih warna atau membersihkan layar.
- Tekan `q` atau `Esc` untuk keluar.

## Shortcut line editor shell

| Shortcut | Fungsi |
|---|---|
| `Up` / `Down` | Navigasi command history |
| `Left` / `Right` | Memindahkan cursor |
| `Ctrl+Left` / `Ctrl+Right` | Memindahkan cursor per kata |
| `Shift+Left` / `Shift+Right` | Memilih teks |
| `Shift+Home` / `Shift+End` | Memilih hingga batas baris |
| `Alt+Backspace` | Menghapus kata sebelumnya |
| `Alt+Delete` | Menghapus kata berikutnya |
| `Ctrl+A` / `Home` | Ke awal baris |
| `Ctrl+E` / `End` | Ke akhir baris |
| `Ctrl+U` | Menghapus seluruh baris |
| `Ctrl+K` | Menghapus dari cursor hingga akhir baris |
| `Ctrl+W` | Menghapus kata sebelumnya |
| `Ctrl+L` | Membersihkan layar dan menggambar ulang input |
| `Ctrl+R` | Reverse history search |
| `Tab` | Command/file completion |
| `Ctrl+C` | Membatalkan input saat ini |

## Alias command

| Alias | Command utama |
|---|---|
| `meminfo` | `free` |
| `netinfo` | `ifconfig` |
| `nslookup` | `dns` |
| `fetch` | `curl` |

Jika command tidak dikenali, shell menampilkan pesan error dan menyarankan penggunaan `help`.
