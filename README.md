# rust-ext
Read Ext2/3 filesystem in Rust...  just for learning Rust :)

```console
$ ext2
Usage:
  ext2 DEVICE COMMAND [ARGUMENTS ...]

Commands:
  cat              Concatenate FILE(s) to standard output.
  df               Show information about the file system.
  hd               Display file contents in hexadecimal.
  ls               List information about the FILEs.
$ ext2 root.dsk ls -l
drwxr-xr-x   19     0     0     1024 Jun  2  2004 .
drwxr-xr-x   19     0     0     1024 Jun  2  2004 ..
drwxr-xr-x    2   506   100     1024 Jun 17  1998 a
drwxr-xr-x    2   506   100     2048 Jun  2  2004 bin
lrwxrwxrwx    1   506   100       13 Jun  2  2004 boot -> /startup/boot
drwxr-xr-x    2   506   100     1024 Mar 14  2002 c
drwxr-xr-x    2   506   100     1024 Mar 16  1998 cdrom
drwxr-xr-x    2   506   100     6144 Jun  2  2004 dev
drwxr-xr-x    5   506   100     2048 Jun  2  2004 etc
drwxr-xr-x    2   506   100     1024 Jul  5  1999 home
drwxr-xr-x    3   506   100     1024 Nov 25  2002 lib
drwxr-xr-x    2   506   100     1024 Aug 11  1999 mnt
drwxr-xr-x    2   506   100     1024 Nov 24  2002 opt
drwxr-xr-x    2   506   100     1024 Oct 22  1999 proc
drwxr-xr-x    2   506   100     1024 Nov 16  2002 root
drwxr-xr-x    2   506   100     1024 Jun  2  2004 sbin
drwxr-xr-x    2   506   100     1024 Jun 18  1999 startup
drwxrwxrwx    2   506   100     1024 Nov 16  2002 tmp
drwxr-xr-x   10   506   100     1024 Nov 11  2002 usr
drwxr-xr-x    7   506   100     1024 Nov  5  2002 var
$ ext2 root.dsk df
Filesystem                        Size           Used      Avail     Use%
root.dsk                       5.12 MB      2.00 MB      3.12 MB      39%
$ ext2 root.dsk cat /etc/passwd
root:/mHZVCrY0dd.s:0:0:root:/root:/bin/bash
admin:qB73YNJOWlNKc:1001:1001::/home/admin:/bin/sh
daemon:lsCgXWEjHODKc:1:1:daemon:/usr/sbin:/bin/sh
bin::2:2:bin:/bin:/bin/nologin
sys::3:3:sys:/dev:/bin/nologin
sync::4:100:sync:/bin:/bin/nologin
lp::7:3:lp:/home/lp:/bin/nologin
mail::8:8::/home/mail:/bin/nologin
ppp::9:9::/tmp:/etc/ppp/ppp_login
httpd:.J/haND/CqCs.:33:33:httpd:/home/httpd:/bin/sh
nobody::65534:65534:nobody:/home:/bin/sh
ftp::100:50::/home/ftp:/bin/false
syslog:!:101:3::/var/log:/bin/nologin
fcron:!:61:61::/tmp:/bin/nologin
michele:g7f17YbPDXw6Y:1003:100::/tmp:/bin/sh
$ ext2 root.dsk hd /usr/lib/libcom_err.so.2.0
00000000  7f 45 4c 46 01 01 01 00 00 00 00 00 00 00 00 00  |.ELF............|
00000010  03 00 03 00 01 00 00 00 e0 08 00 00 34 00 00 00  |............4...|
00000020  24 12 00 00 00 00 00 00 34 00 20 00 03 00 28 00  |$.......4. ...(.|
00000030  19 00 18 00 01 00 00 00 00 00 00 00 00 00 00 00  |................|
00000040  00 00 00 00 65 0e 00 00 65 0e 00 00 05 00 00 00  |....e...e.......|
00000050  00 10 00 00 01 00 00 00 68 0e 00 00 68 1e 00 00  |........h...h...|
00000060  68 1e 00 00 1c 01 00 00 58 01 00 00 06 00 00 00  |h.......X.......|
00000070  00 10 00 00 02 00 00 00 e4 0e 00 00 e4 1e 00 00  |................|
00000080  e4 1e 00 00 a0 00 00 00 a0 00 00 00 06 00 00 00  |................|
00000090  04 00 00 00 25 00 00 00 34 00 00 00 00 00 00 00  |....%...4.......|
000000a0  00 00 00 00 2d 00 00 00 00 00 00 00 1a 00 00 00  |....-...........|
000000b0  00 00 00 00 00 00 00 00 19 00 00 00 33 00 00 00  |............3...|
000000c0  25 00 00 00 00 00 00 00 29 00 00 00 00 00 00 00  |%.......).......|
000000d0  00 00 00 00 27 00 00 00 00 00 00 00 00 00 00 00  |....'...........|
...
```
