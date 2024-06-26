# Progress
- [X] 在未使用标准库的情况下在Qemu屏幕上成功打印“Hello World!”(向bootloader提供的缓冲区0xb8000写入"Hello World!")
- [X] 封装了上一步的打印操作，自定义了println函数
- [X] 实现测试框架 以使用“cargo test”测试代码
- [X] 实现中断向量表，使CPU可以捕获普通异常，如：除零，断点
- [X] 实现double fault handler(启用栈指针切换功能 使用IST序号切换到另一个特殊的栈 使其在栈溢出时也可正常工作)
- [X] 硬件中断实现(PIC 添加了定时器，键盘中断)
- [X] bootloader：引入分页机制(四级页表 48位：四页索引 三页索引 二页索引 一页索引 偏移)
- [X] 完整页表实现(映射完整的物理内存)
    - 递归页表
        - 映射1级表 递归1次4级表(即第一次访问是从4级到4级)
        - 映射2级表 递归2次4级表(即第一二次访问都是从4级到4级)
        - 映射3级表 递归3次4级表(即第一二三次访问都是从4级到4级)
        - 映射4级表 递归4次4级表(即第一二三四次访问都是从4级到4级)
    - 向地址写入数据时
        - 检查是否分配页
        - 已分配页则直接写入
        - 未分配则查找未使用的地址空间建立页表映射再写入数据
- [X] 堆内存分配实现
    - 基础概览
        - 静态变量
            - 所有权不清
            - 贸然改变值会导致未定义行为（以互斥类型修饰可避免）
            - 生命周期与程序运行时间相当
            - 存储在静态变量区 (.bss)
            - 内存大小固定
        - 局部变量
            - 所有权清晰
            - 可以任意改变值
            - 生命周期短暂 随声明函数一起消失
            - 存储在栈区 (.text)
            - 内存大小固定
        - 堆变量
            - 可任意改变值
            - 调用deallocate()后生命周期结束
            - 内存大小可任意由allocate()指定
    - 分配器设计
        - [X] linked list allocation 在页表中开辟一块大内存以链表的形式向需要使用的程序分配内存
        - [X] bump allocation 类似于数组顺序分配内存
        - [X] fixed-size block allocation 
- [ ] 系统多进程实现 async/await

# Acknowledge
[blog OS](https://os.phil-opp.com/)
