import os

# 对当前项目的简单描述
preprompt = "这是一个简单的用Rust编写的操作系统内核的项目的所有代码。"

ignored_files = [ "./LICENSE", "./prompt-generator.py", "./os/Cargo.lock", "./user/Cargo.lock", "./output.log", "./sdcard.img", "./results.json", "./log", "./sbi-qemu", "./kernel-qemu" ]
ignored_directories = [ "./os/target", "./user/target" , "./bootloader" , "./.vscode", "./.git", "./.github", "./test", "./thirdparty", "./docs"]

languages = [
    [".rs", "rust"],
    [".md", "markdown"],
    [".S", "asm"],
    [".s", "asm"],
]

def get_language(file_path: str):
    for l in languages:
        if file_path.endswith(l[0]):
            return l[1]
    return ""

def read_file(file_path: str) -> str:
    with open(file_path, 'r') as file:
        return file.read()

def handle_single_file(file_path: str):
    language = get_language(file_path)
    print('- ' + file_path)
    # Begin code snippet
    print('```' + language)
    print(read_file(file_path))
    # End code snippet
    print('```')
    print()

def handle_files():
    for root, dirs, files in os.walk('.'):
        # 忽略指定的目录./user/src/syscall.rs
        dirs[:] = [d for d in dirs if os.path.join(root, d) not in ignored_directories]

        for file in files:
            file_path = os.path.join(root, file)
            if file_path not in ignored_files:
                # print(file_path)
                handle_single_file(file_path)

if __name__ == "__main__":
    print(preprompt)
    handle_files()
