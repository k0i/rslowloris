
This repository is still under development!!!!!!

# rslowloris
> Slowloris is a type of denial of service attack tool which allows a single machine to take down another machine's web server with minimal bandwidth and side effects on unrelated services and ports.

![output (online-video-cutter com) (1)](https://user-images.githubusercontent.com/100127291/205270513-17035611-da55-4562-bf38-9cbee02cc25b.gif)


A rust implementation of slowloris.

# Roadmap
- SOCK5 Proxy

# Install

Please refer to [release](https://github.com/k0i/rslowloris/releases) page.

# Usage

```
rslowloris ${target ip or domain}


Flags:
        -v, --verbose                   : Verbose logging flag
        -s, --socket <int>              : socket count
        -p, --port <int>                : port
        -ho, --httponly                 : use http connection
        -pf, --proxy_file_path <string> : proxy file path
        -h, --help                      : Show help

Version:
```

### Default TLS
Unless otherwise specified, it connects with TLS (port 443).
If you want to attack by HTTP connection, specify -ho (--httponly) flag. 

### Proxy File
If you want to use SOCK5 Proxy, please specify the filepath that contains proxy servers' address using `pf` flag.

**example_proxy_list.txt**
```
111.111.111.111:7669
222.222.222.222:2345
```
