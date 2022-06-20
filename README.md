# NBundle
Simple bundler, for Tamper Monkey plugins and BetterNCM plugins

一个简单的可以将多个文件转换为单文件的程序。支持实时更新。

速度极快，调试顺畅。

# 何时使用？
当你正在制作一个复杂的Tampermonkey插件，想要进行分文件开发，使用less等语言。

当你在写一个BetterDiscord/BetterNCM插件，想要将html文件独立出来

# 使用
## 1.配置命令行
```
NBundle 1.0
MicroBlock
Bundle everything to one javascript file.

USAGE:
    nbundle [OPTIONS] --dir <Dir> --output <Output>

OPTIONS:
    -d, --dir <Dir>          Sets the monitoring dir. | 设置监控的文件夹
    -h, --help               Print help information | 打印帮助信息
    -m, --main <Main>        Sets the main file. | 设置入口文件（默认为main.js）
    -o, --output <Output>    Sets the output file. | 设置输出文件
    -V, --version            Print version information | 打印版本信息
```

程序将会监控指定文件夹下的所有变动。

如：将当前目录下，以`main.js`为入口点，打包为上级目录下`livesongplayer-bundled.js`，写为：

```
nbundle.exe --dir ./ --output ../livesongplayer-bundled.js
```
## 2.语法
### 字符串型
例：`"#require /www/foo.js#"`

该种类型将会保持字符串类型。

如：

编译前：
```js
// main.js
console.log("#require /foo.txt#")

// foo.txt
bar
```
编译后：
```js
// bundled.js
window['nbundle-build-xx']={"nbundle-string-match-xx":"bar"}
console.log(window['nbundle-build-xx']["nbundle-string-match-xx"])
```
或
```js
console.log("bar")
```

### 注释型
该种类型将不保留注释

如：

编译前：
```js
// main.js
console.log("Main.js")
/*#require /a.js#*/

// a.js
console.log("NBundle!");
```
编译后：
```js
// bundled.js
console.log("Main.js")
(function(){console.log("NBundle!");})()
```

## 3.指令
### require
 - 正常的引用指令
 - 可能视情况进行优化
 - 例：
 `/*#require /a.js#*/`->`(function(){console.log("NBundle!");})()`
### raw_require
 - 直接替换
 - 例：
 `/*#raw_require /a.js#*/`->`console.log("NBundle!");`

## 4.使用例
https://github.com/MicroCBer/NeteaseLiveSongPlayer
