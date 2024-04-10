# Original intention

watch one token price change on app menu bar .

avoid go to fanince app , because the BOSS is watching YOU.

btw , price is updated every 3 seconds !

![TO THE MOON!!!](public/barview.png)

## How To Use ？ (JUST WATCH)

[![](https://i.ytimg.com/vi/MpTIEvQGSZU/hqdefault.jpg)](https://www.youtube.com/watch?v=MpTIEvQGSZU)

## Developer Getting Started

download this project , run dev after install deps.

follow [tauri quickstart guide](https://tauri.app/zh-cn/v1/guides/getting-started/prerequisites) before run this project .

### Dev

```
$ git clone https://github.com/lifecoder1988/tauri-watch-coin.git
$ cd ./tauri-watch-coin
$ yarn install
$ yarn tauri dev

```

### Build

```
$ git clone https://github.com/lifecoder1988/tauri-watch-coin.git
$ cd ./tauri-watch-coin
$ yarn install
$ yarn tauri build

```

## OS Supported

| OS      | Status             |
| ------- | ------------------ |
| Windows | UNKOWN             |
| MacOS   | :white_check_mark: |

## FAQ

### 1. Can't open app after install

because this app is not code signed by apple right now . you can run this command to skip this issue . (because lack of money)

```
sudo xattr -d com.apple.quarantine /Applications/watchcoin.app
```

### 2. Where to Download Release App

recently , you can download app for macos only . i am study hard to use github action to build app for other OS.

[download app for mac M1 M2 M3 MXX ](https://github.com/lifecoder1988/tauri-watch-coin/blob/main/pkg/watchcoin_0.0.1_aarch64.dmg)
