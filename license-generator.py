# 注意：引用新库后请务必重新运行脚本

import os

def scan_licenses(thirdparty_dir):
    license_text = ""

    # 获取 thirdparty 目录下的所有子目录
    directories = [d for d in os.listdir(thirdparty_dir) if os.path.isdir(os.path.join(thirdparty_dir, d))]

    # 遍历每个子目录
    for directory in directories:
        found = False

        dir_path = os.path.join(thirdparty_dir, directory)
        dir_name = os.path.basename(dir_path)

        license_text += f"== {dir_name} ==\n"
        license_text += "----------------\n\n"

        # 扫描目录根目录下的文件，查找包含 "LICENSE" 字样的文件
        for file_name in os.listdir(dir_path):
            if "LICENSE" in file_name.upper() or "LICENCE" in file_name.upper():
                file_path = os.path.join(dir_path, file_name)

                file_name = os.path.basename(file_path)

                license_text += f"{file_name}\n"
                license_text += "----------------\n\n"

                # 读取文件内容并追加到输出字符串中
                with open(file_path, 'r', encoding='utf-8') as file:
                    license_text += file.read() + "\n\n"

                found = True

        if not found:
            print(f"WARNING: No license file found in {dir_name}")

    return license_text

# 示例使用
thirdparty_dir = "thirdparty"
output = scan_licenses(thirdparty_dir)

# 保存到`thirdpartylegalnotices.md`
with open("thirdpartylegalnotices.md", 'w', encoding='utf-8') as file:
    file.write(output)
