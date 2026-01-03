# 用于开发过程中快速搭建验证平台
为了在cocotb中应用，快速搭建测试平台，简化收集filelist的过程。所以将其编译为python 库，可以直接在python 脚本中调用生成List[String] 类型。
并且注意了生成的filelist的顺序，底层模块放在list的上方，顶层模块放在list 底部。便于vcs 编译工具直接读取。

## 使用方法：
将编译生成的libmex.so 文件修改名字为mex.so, 并将其放置到python 脚本的执行目录下即可。
```python
import mex
file_list = mex.get(hdl_root_dir, top_module_name)
```
