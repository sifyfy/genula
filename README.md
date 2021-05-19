# genula

## 何をするもの？

IPv6 ULAのグローバルIDを生成してprefixをユーザーに提示します。

## 想定する動作環境

* Linux系OS
* Windows 10
* x86_64 CPU Architecture
  * 特にCPUに縛られる要素は無いはずだけど、一応。

## 使い方

### GUIモード

未実装。

実行ファイルにCLIオプションを付けずに実行するとGUIモードで起動します。

### CLIモード

Usage 1: `genula --use-mac-address-of-this-node`

Usage 2: `genula --mac-address MAC_ADDRESS`

Usage 3: `genula --unique-identifier UNIQUE_IDENTIFIER_OF_THIS_NODE`
