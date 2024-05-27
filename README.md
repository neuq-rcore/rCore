# neuqOS 技术文档

![NEUQ](docs/assets/neuq.jpg)

## 所有文档

请查看 [文档总目录](docs/content.md)

---

## 自动化测试

[![CodeFactor](https://www.codefactor.io/repository/github/neuq-rcore/rcore/badge)](https://www.codefactor.io/repository/github/neuq-rcore/rcore)

[![Continuous Integration](https://github.com/neuq-rcore/rCore/actions/workflows/ci.yml/badge.svg)](https://github.com/neuq-rcore/rCore/actions/workflows/ci.yml)

[![Sync to GitLab](https://github.com/neuq-rcore/rCore/actions/workflows/mirror.yml/badge.svg)](https://github.com/neuq-rcore/rCore/actions/workflows/mirror.yml)

[![OJ Simulation](https://github.com/neuq-rcore/rCore/actions/workflows/oj.yml/badge.svg)](https://github.com/neuq-rcore/rCore/actions/workflows/oj.yml)

## 准备工作

### 构建

```shell
# or simply run `make`
make build
```

### 运行

```shell
make run
```

### 测试

#### 本地测试

在仓库根目录执行：

```shell
make test
```

这将模拟比赛的评测环境，首先执行 `make all` ，然后使用要求的 Qemu 启动参数挂载测试样例并运行内核进行测试。运行结束后，测试脚本 `test/visualize_result.py` 将会生成测试结果的可视化报告。

下面是生成的可视化报告的一个例子：

![visual_report.png](docs/assets/visual_report.png)

对于每个测试，

- **Skiped** 表示测试未进行，或者该测试样例未输出结果就被内核杀死
- **Failed** 表示测试已经执行，且有输出，但是测试结果不符合预期
- **Passed** 表示测试通过，输出结果符合预期

其中，`[x/y]` 表示单个测试的结果，`x` 为测试通过的测试点数量，`y` 为该测试的测试点总数。

最后会给出测试的总体结果和得分。

#### 持续集成测试

对于submit分支下的每次提交（将来会合并到 `main` 分支），都会有一个 GitHub Actions workflow 自动运行测试。测试过程基本符合上述本地测试的流程，但是会在测试结束后将测试结果，Qemu输出和测试脚本对输出的判断结果都上传到 GitHub Actions 的 artifacts 中，以便查看详细的测试结果。同时，可视化脚本也会运行，无需下载 artifacts 即可查看测试结果。

### 调试

### 命令行界面

#### 通过 GDB 启动 QEMU 模拟器

```shell
make debug
```

#### 连接至 GDB 服务

```shell
make connect
```

### VSCode

在 VSCode 中打开项目，按下 <kbd>F5</kbd> 进行调试。

## 完成情况

| 内核模块 | 完成情况 | 系统调用 |
| :------- | -------- | -------- |
|          |          |          |
|          |          |          |
|          |          |          |
|          |          |          |

## 参赛队员

徐才益

薛丁豪

白聪

## 参考文档

- [rCore-Tutorial-Book-v3 3.6.0-alpha.1 文档](https://rcore-os.cn/rCore-Tutorial-Book-v3/index.html)

- [plctlab/riscv-operating-system-mooc: 《从头写一个RISC-V OS》课程配套的资源](https://github.com/plctlab/riscv-operating-system-mooc)

- [xsp-daily-work/暑期rcore实验笔记 at master · xushanpu123/xsp-daily-work](https://github.com/xushanpu123/xsp-daily-work/tree/master/暑期rcore实验笔记)

- [Introduction · GitBook](https://nju-projectn.github.io/ics-pa-gitbook/ics2024/)

## 许可

MIT

## 联系我们

如若有问题欢迎与本团队联系，我们会在第一时间给您回复，邮箱cai1hsu@outlook.com，欢迎您踊跃参与

## 知识产权与学术诚信

- 本团队仓库在初赛期间全过程保持开源状态

- 本团队全程严守诚信，不存在抄袭谎报现象
